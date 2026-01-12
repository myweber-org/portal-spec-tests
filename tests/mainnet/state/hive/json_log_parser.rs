use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line_content) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line_content, e),
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect();
        Ok(filtered)
    }

    pub fn filter_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect();
        Ok(filtered)
    }

    pub fn count_entries_by_component(&self) -> Result<std::collections::HashMap<String, usize>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let mut counts = std::collections::HashMap::new();

        for entry in entries {
            *counts.entry(entry.component).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let log_lines = vec![
            r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","component":"database","metadata":{"error_code":1001}}"#,
            r#"{"timestamp":"2024-01-15T10:31:00Z","level":"WARN","message":"High memory usage","component":"memory","metadata":{"usage_percent":85}}"#,
            r#"{"timestamp":"2024-01-15T10:32:00Z","level":"INFO","message":"User login successful","component":"auth","metadata":null}"#,
            r#"{"timestamp":"2024-01-15T10:33:00Z","level":"ERROR","message":"API timeout","component":"api","metadata":{"timeout_seconds":30}}"#,
            r#"{"timestamp":"2024-01-15T10:34:00Z","level":"INFO","message":"Cache cleared","component":"cache","metadata":null}"#,
        ];

        for line in log_lines {
            writeln!(file, "{}", line).unwrap();
        }
        file
    }

    #[test]
    fn test_parse_log_file() {
        let file = create_test_log_file();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_filter_by_level() {
        let file = create_test_log_file();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let errors = parser.filter_by_level(LogLevel::ERROR).unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.level == LogLevel::ERROR));
    }

    #[test]
    fn test_filter_by_time_range() {
        let file = create_test_log_file();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let start = Utc.with_ymd_and_hms(2024, 1, 15, 10, 31, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 15, 10, 33, 0).unwrap();
        let filtered = parser.filter_by_time_range(start, end).unwrap();
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_count_entries_by_component() {
        let file = create_test_log_file();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let counts = parser.count_entries_by_component().unwrap();
        assert_eq!(counts.get("database"), Some(&1));
        assert_eq!(counts.get("api"), Some(&1));
    }
}