use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
}

pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            self.entries.push(entry);
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_error_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.level == LogLevel::ERROR)
            .count()
    }

    pub fn get_component_errors(&self, component: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == LogLevel::ERROR && entry.component == component)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","component":"boot"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Failed to connect","component":"network"}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","message":"High memory usage","component":"monitor"}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let mut parser = LogParser::new();
        parser.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(parser.total_entries(), 3);
        assert_eq!(parser.get_error_count(), 1);
        
        let errors = parser.filter_by_level(LogLevel::ERROR);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].component, "network");
    }

    #[test]
    fn test_component_filtering() {
        let mut parser = LogParser::new();
        parser.entries.push(LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: LogLevel::ERROR,
            message: "Test error".to_string(),
            component: "database".to_string(),
        });
        
        let db_errors = parser.get_component_errors("database");
        assert_eq!(db_errors.len(), 1);
        
        let network_errors = parser.get_component_errors("network");
        assert_eq!(network_errors.len(), 0);
    }
}use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogSeverity {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub severity: LogSeverity,
    pub service: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct LogParser {
    min_severity: LogSeverity,
}

impl LogParser {
    pub fn new(min_severity: LogSeverity) -> Self {
        LogParser { min_severity }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    if self.should_include(&entry.severity) {
                        entries.push(entry);
                    }
                }
                Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
            }
        }

        Ok(entries)
    }

    fn should_include(&self, severity: &LogSeverity) -> bool {
        let severity_value = match severity {
            LogSeverity::DEBUG => 0,
            LogSeverity::INFO => 1,
            LogSeverity::WARN => 2,
            LogSeverity::ERROR => 3,
            LogSeverity::CRITICAL => 4,
        };

        let min_value = match self.min_severity {
            LogSeverity::DEBUG => 0,
            LogSeverity::INFO => 1,
            LogSeverity::WARN => 2,
            LogSeverity::ERROR => 3,
            LogSeverity::CRITICAL => 4,
        };

        severity_value >= min_value
    }

    pub fn filter_by_service(&self, entries: &[LogEntry], service: &str) -> Vec<LogEntry> {
        entries
            .iter()
            .filter(|entry| entry.service == service)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_and_filter() {
        let log_data = r#"
            {"timestamp": "2023-10-01T12:00:00Z", "severity": "INFO", "service": "api", "message": "Request received"}
            {"timestamp": "2023-10-01T12:00:01Z", "severity": "ERROR", "service": "db", "message": "Connection failed", "metadata": {"attempt": 3}}
            {"timestamp": "2023-10-01T12:00:02Z", "severity": "DEBUG", "service": "api", "message": "Processing data"}
            {"timestamp": "2023-10-01T12:00:03Z", "severity": "WARN", "service": "cache", "message": "High memory usage"}
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(LogSeverity::WARN);
        let entries = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].severity, LogSeverity::ERROR);
        assert_eq!(entries[1].severity, LogSeverity::WARN);

        let api_logs = parser.filter_by_service(&entries, "api");
        assert_eq!(api_logs.len(), 0);

        let db_logs = parser.filter_by_service(&entries, "db");
        assert_eq!(db_logs.len(), 1);
        assert_eq!(db_logs[0].service, "db");
    }
}