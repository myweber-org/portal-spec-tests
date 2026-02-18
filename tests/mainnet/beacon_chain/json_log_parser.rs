use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
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
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
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

    pub fn count_entries(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","component":"boot"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Disk full","component":"storage"}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","message":"High memory usage","component":"memory"}
{"timestamp":"2024-01-15T10:33:00Z","level":"INFO","message":"Backup completed","component":"backup"}"#;
        write!(file, "{}", log_data).unwrap();
        file
    }

    #[test]
    fn test_parse_log_entries() {
        let file = create_test_log();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();
        assert_eq!(entries.len(), 4);
    }

    #[test]
    fn test_filter_error_logs() {
        let file = create_test_log();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let errors = parser.filter_by_level(LogLevel::ERROR).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Disk full");
    }

    #[test]
    fn test_count_entries() {
        let file = create_test_log();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let count = parser.count_entries().unwrap();
        assert_eq!(count, 4);
    }
}