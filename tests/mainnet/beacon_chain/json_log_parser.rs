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
}use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
    Unknown,
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "info" => LogLevel::Info,
            "warning" => LogLevel::Warning,
            "error" => LogLevel::Error,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Unknown,
        }
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub component: Option<String>,
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
            if let Ok(parsed) = self.parse_line(&line) {
                entries.push(parsed);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp = json_value["timestamp"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let level_str = json_value["level"]
            .as_str()
            .unwrap_or("unknown");
        let level = LogLevel::from(level_str);

        let message = json_value["message"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let component = json_value["component"]
            .as_str()
            .map(|s| s.to_string());

        Ok(LogEntry {
            timestamp,
            level,
            message,
            component,
        })
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let all_entries = self.parse()?;
        let filtered: Vec<LogEntry> = all_entries
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect();
        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_log_entries() {
        let log_data = r#"{"timestamp": "2023-10-01T12:00:00Z", "level": "info", "message": "System started", "component": "boot"}
{"timestamp": "2023-10-01T12:01:00Z", "level": "error", "message": "Disk full", "component": "storage"}
{"timestamp": "2023-10-01T12:02:00Z", "level": "warning", "message": "High memory usage"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, LogLevel::Info);
        assert_eq!(entries[1].level, LogLevel::Error);
        assert_eq!(entries[2].level, LogLevel::Warning);
    }

    #[test]
    fn test_filter_error_logs() {
        let log_data = r#"{"timestamp": "2023-10-01T12:00:00Z", "level": "info", "message": "System started"}
{"timestamp": "2023-10-01T12:01:00Z", "level": "error", "message": "Disk full"}
{"timestamp": "2023-10-01T12:02:00Z", "level": "error", "message": "Network timeout"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let error_logs = parser.filter_by_level(LogLevel::Error).unwrap();

        assert_eq!(error_logs.len(), 2);
        assert!(error_logs.iter().all(|entry| entry.level == LogLevel::Error));
    }
}