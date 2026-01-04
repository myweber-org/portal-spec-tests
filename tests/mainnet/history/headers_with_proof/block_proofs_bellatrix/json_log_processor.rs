use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

impl LogLevel {
    pub fn from_str(level: &str) -> Option<Self> {
        match level.to_uppercase().as_str() {
            "DEBUG" => Some(LogLevel::DEBUG),
            "INFO" => Some(LogLevel::INFO),
            "WARN" => Some(LogLevel::WARN),
            "ERROR" => Some(LogLevel::ERROR),
            "CRITICAL" => Some(LogLevel::CRITICAL),
            _ => None,
        }
    }

    pub fn severity(&self) -> u8 {
        match self {
            LogLevel::DEBUG => 1,
            LogLevel::INFO => 2,
            LogLevel::WARN => 3,
            LogLevel::ERROR => 4,
            LogLevel::CRITICAL => 5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    include_metadata: bool,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel, include_metadata: bool) -> Self {
        LogProcessor {
            min_level,
            include_metadata,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if let Some(entry) = self.parse_line(&line) {
                if entry.level.severity() >= self.min_level.severity() {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        serde_json::from_str::<LogEntry>(line).ok().map(|mut entry| {
            if !self.include_metadata {
                entry.metadata.clear();
            }
            entry
        })
    }

    pub fn filter_by_level(&self, entries: &[LogEntry], level: LogLevel) -> Vec<LogEntry> {
        entries
            .iter()
            .filter(|entry| entry.level == level)
            .cloned()
            .collect()
    }

    pub fn extract_timestamps(&self, entries: &[LogEntry]) -> Vec<String> {
        entries.iter().map(|entry| entry.timestamp.clone()).collect()
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
        let mut counts = HashMap::new();
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::INFO));
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::ERROR));
        assert_eq!(LogLevel::from_str("UNKNOWN"), None);
    }

    #[test]
    fn test_log_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","metadata":{"user":"admin"}}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Connection failed","metadata":{"ip":"192.168.1.1"}}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let processor = LogProcessor::new(LogLevel::INFO, true);
        let entries = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::INFO);
        assert_eq!(entries[1].level, LogLevel::ERROR);
    }

    #[test]
    fn test_level_filtering() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: LogLevel::INFO,
                message: "Test info".to_string(),
                metadata: HashMap::new(),
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: LogLevel::ERROR,
                message: "Test error".to_string(),
                metadata: HashMap::new(),
            },
        ];

        let processor = LogProcessor::new(LogLevel::DEBUG, false);
        let filtered = processor.filter_by_level(&entries, LogLevel::ERROR);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, LogLevel::ERROR);
    }
}