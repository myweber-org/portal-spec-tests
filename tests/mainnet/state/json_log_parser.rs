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
}