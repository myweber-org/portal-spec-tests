
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    service_filter: Option<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            service_filter: None,
        }
    }

    pub fn with_service_filter(mut self, service: &str) -> Self {
        self.service_filter = Some(service.to_string());
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.should_include(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        if self.level_priority(&entry.level) < self.level_priority(&self.min_level) {
            return false;
        }

        if let Some(ref service_filter) = self.service_filter {
            if entry.service != *service_filter {
                return false;
            }
        }

        true
    }

    fn level_priority(&self, level: &LogLevel) -> u8 {
        match level {
            LogLevel::DEBUG => 1,
            LogLevel::INFO => 2,
            LogLevel::WARN => 3,
            LogLevel::ERROR => 4,
            LogLevel::CRITICAL => 5,
        }
    }
}

pub fn count_logs_by_level(entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
    let mut counts = HashMap::new();
    for entry in entries {
        *counts.entry(entry.level.clone()).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed","metadata":{"user_id":"123"}}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"db","message":"Connection failed","metadata":{"retry_count":"3"}}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let processor = LogProcessor::new(LogLevel::INFO);
        let entries = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].service, "api");
        assert_eq!(entries[1].level, LogLevel::ERROR);
    }

    #[test]
    fn test_level_filtering() {
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"DEBUG","service":"api","message":"Debug info","metadata":{}}
{"timestamp":"2024-01-15T10:31:00Z","level":"WARN","service":"api","message":"Warning","metadata":{}}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let processor = LogProcessor::new(LogLevel::WARN);
        let entries = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, LogLevel::WARN);
    }
}