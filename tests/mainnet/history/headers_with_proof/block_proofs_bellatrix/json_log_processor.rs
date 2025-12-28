
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug)]
pub enum LogError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
    InvalidFormat(String),
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogError::IoError(msg) => write!(f, "IO error: {}", msg),
            LogError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LogError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            LogError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl Error for LogError {}

impl From<std::io::Error> for LogError {
    fn from(error: std::io::Error) -> Self {
        LogError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for LogError {
    fn from(error: serde_json::Error) -> Self {
        LogError::ParseError(error.to_string())
    }
}

pub struct LogProcessor {
    pub entries: Vec<LogEntry>,
    pub stats: ProcessingStats,
}

#[derive(Debug, Default, Serialize)]
pub struct ProcessingStats {
    pub total_lines: usize,
    pub parsed_successfully: usize,
    pub parse_failures: usize,
    pub validation_failures: usize,
    pub level_distribution: HashMap<String, usize>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            stats: ProcessingStats::default(),
        }
    }

    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            self.stats.total_lines += 1;
            let line = line_result?;

            match self.parse_log_line(&line) {
                Ok(entry) => {
                    if self.validate_entry(&entry) {
                        self.stats.parsed_successfully += 1;
                        *self.stats.level_distribution.entry(entry.level.clone()).or_insert(0) += 1;
                        self.entries.push(entry);
                    } else {
                        self.stats.validation_failures += 1;
                    }
                }
                Err(_) => {
                    self.stats.parse_failures += 1;
                }
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Result<LogEntry, LogError> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp = json_value["timestamp"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing timestamp".to_string()))?
            .to_string();

        let level = json_value["level"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing level".to_string()))?
            .to_string();

        let service = json_value["service"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing service".to_string()))?
            .to_string();

        let message = json_value["message"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing message".to_string()))?
            .to_string();

        let metadata = if let Some(obj) = json_value["metadata"].as_object() {
            let mut map = HashMap::new();
            for (key, value) in obj {
                map.insert(key.clone(), value.clone());
            }
            map
        } else {
            HashMap::new()
        };

        Ok(LogEntry {
            timestamp,
            level,
            service,
            message,
            metadata,
        })
    }

    fn validate_entry(&self, entry: &LogEntry) -> bool {
        !entry.timestamp.is_empty()
            && !entry.level.is_empty()
            && !entry.service.is_empty()
            && !entry.message.is_empty()
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    pub fn get_stats(&self) -> &ProcessingStats {
        &self.stats
    }

    pub fn export_stats_json(&self) -> Result<String, LogError> {
        serde_json::to_string(&self.stats).map_err(|e| LogError::ParseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log_line() {
        let processor = LogProcessor::new();
        let log_line = r#"{
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "INFO",
            "service": "api-gateway",
            "message": "Request processed successfully",
            "metadata": {
                "request_id": "abc123",
                "duration_ms": 150
            }
        }"#;

        let result = processor.parse_log_line(log_line);
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.service, "api-gateway");
        assert_eq!(entry.message, "Request processed successfully");
        assert_eq!(entry.metadata.get("request_id").unwrap().as_str().unwrap(), "abc123");
    }

    #[test]
    fn test_parse_invalid_log_line() {
        let processor = LogProcessor::new();
        let log_line = r#"{"invalid": "json"}"#;
        let result = processor.parse_log_line(log_line);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "service": "database", "message": "Connection failed"}
{"timestamp": "2024-01-15T10:31:00Z", "level": "WARN", "service": "cache", "message": "High memory usage"}
{"timestamp": "2024-01-15T10:32:00Z", "level": "INFO", "service": "api", "message": "Health check passed"}"#;
        
        writeln!(temp_file, "{}", log_data).unwrap();
        
        let mut processor = LogProcessor::new();
        let result = processor.process_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.stats.total_lines, 3);
        assert_eq!(processor.stats.parsed_successfully, 3);
    }

    #[test]
    fn test_filter_by_level() {
        let mut processor = LogProcessor::new();
        processor.entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: "ERROR".to_string(),
                service: "database".to_string(),
                message: "Connection failed".to_string(),
                metadata: HashMap::new(),
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: "INFO".to_string(),
                service: "api".to_string(),
                message: "Request processed".to_string(),
                metadata: HashMap::new(),
            },
            LogEntry {
                timestamp: "2024-01-15T10:32:00Z".to_string(),
                level: "ERROR".to_string(),
                service: "cache".to_string(),
                message: "Cache miss".to_string(),
                metadata: HashMap::new(),
            },
        ];

        let errors = processor.filter_by_level("ERROR");
        assert_eq!(errors.len(), 2);
        
        let infos = processor.filter_by_level("INFO");
        assert_eq!(infos.len(), 1);
    }
}