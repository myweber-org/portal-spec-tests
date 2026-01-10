
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, PartialEq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

#[derive(Debug, Deserialize)]
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

    pub fn parse_with_filter(&self, min_level: LogLevel) -> Result<Vec<LogEntry>, String> {
        let path = Path::new(&self.file_path);
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);

        let mut filtered_entries = Vec::new();
        let level_order = vec![
            LogLevel::DEBUG,
            LogLevel::INFO,
            LogLevel::WARN,
            LogLevel::ERROR,
            LogLevel::CRITICAL,
        ];

        let min_level_index = level_order
            .iter()
            .position(|l| l == &min_level)
            .unwrap_or(0);

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line_content)
                .map_err(|e| format!("Line {} parse error: {}", line_num + 1, e))?;

            let entry_level_index = level_order
                .iter()
                .position(|l| l == &entry.level)
                .unwrap_or(0);

            if entry_level_index >= min_level_index {
                filtered_entries.push(entry);
            }
        }

        Ok(filtered_entries)
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> std::collections::HashMap<LogLevel, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for entry in entries {
            *counts.entry(entry.level).or_insert(0) += 1;
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
    fn test_parse_with_filter() {
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "System started", "component": "boot"}
{"timestamp": "2024-01-15T10:31:00Z", "level": "DEBUG", "message": "Memory usage: 45%", "component": "monitor"}
{"timestamp": "2024-01-15T10:32:00Z", "level": "ERROR", "message": "Disk full", "component": "storage"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse_with_filter(LogLevel::INFO).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|e| e.level == LogLevel::INFO));
        assert!(result.iter().any(|e| e.level == LogLevel::ERROR));
        assert!(!result.iter().any(|e| e.level == LogLevel::DEBUG));
    }

    #[test]
    fn test_count_by_level() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: LogLevel::INFO,
                message: "Test message 1".to_string(),
                component: Some("test".to_string()),
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: LogLevel::ERROR,
                message: "Test message 2".to_string(),
                component: None,
            },
            LogEntry {
                timestamp: "2024-01-15T10:32:00Z".to_string(),
                level: LogLevel::INFO,
                message: "Test message 3".to_string(),
                component: Some("test".to_string()),
            },
        ];

        let parser = LogParser::new("dummy.log");
        let counts = parser.count_by_level(&entries);

        assert_eq!(counts.get(&LogLevel::INFO), Some(&2));
        assert_eq!(counts.get(&LogLevel::ERROR), Some(&1));
        assert_eq!(counts.get(&LogLevel::DEBUG), None);
    }
}