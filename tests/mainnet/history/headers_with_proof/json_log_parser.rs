
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: Option<String>,
    filter_fields: HashMap<String, Value>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            filter_fields: HashMap::new(),
        }
    }

    pub fn set_min_level(&mut self, level: &str) -> &mut Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    pub fn add_filter(&mut self, key: &str, value: Value) -> &mut Self {
        self.filter_fields.insert(key.to_string(), value);
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_lowercase();

        if let Some(min_level) = &self.min_level {
            if !self.level_passes(&level, min_level) {
                return Err("Log level below minimum threshold".into());
            }
        }

        let message = json_value.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        for (filter_key, filter_value) in &self.filter_fields {
            if let Some(entry_value) = fields.get(filter_key) {
                if entry_value != filter_value {
                    return Err("Field filter mismatch".into());
                }
            } else {
                return Err("Required field not found".into());
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn level_passes(&self, entry_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error", "fatal"];
        let entry_idx = levels.iter().position(|&l| l == entry_level);
        let min_idx = levels.iter().position(|&l| l == min_level);

        match (entry_idx, min_idx) {
            (Some(e), Some(m)) => e >= m,
            _ => false,
        }
    }
}

impl LogEntry {
    pub fn format(&self, show_fields: bool) -> String {
        let mut output = format!("[{}] {}: {}", self.timestamp, self.level.to_uppercase(), self.message);
        
        if show_fields && !self.fields.is_empty() {
            output.push_str(" | ");
            for (key, value) in &self.fields {
                output.push_str(&format!("{}={:?} ", key, value));
            }
        }
        
        output.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_log() {
        let parser = LogParser::new();
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Service started","service":"api","duration":150}"#;
        
        let entry = parser.parse_line(log_line).unwrap();
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "Service started");
        assert_eq!(entry.fields.get("service").unwrap().as_str().unwrap(), "api");
    }

    #[test]
    fn test_level_filtering() {
        let mut parser = LogParser::new();
        parser.set_min_level("warn");
        
        let info_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Test"}"#;
        let warn_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"WARN","message":"Warning"}"#;
        
        assert!(parser.parse_line(info_log).is_err());
        assert!(parser.parse_line(warn_log).is_ok());
    }
}use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid log format at line {0}")]
    InvalidFormat(usize),
}

pub struct LogParser {
    field_filters: HashMap<String, String>,
    level_filter: Option<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            field_filters: HashMap::new(),
            level_filter: None,
        }
    }

    pub fn add_field_filter(&mut self, key: &str, value: &str) {
        self.field_filters.insert(key.to_string(), value.to_string());
    }

    pub fn set_level_filter(&mut self, level: &str) {
        self.level_filter = Some(level.to_string());
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<Value>, LogParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let json_value: Value = serde_json::from_str(&line)
                .map_err(|e| LogParseError::Json(e))?;

            if self.matches_filters(&json_value) {
                results.push(json_value);
            }
        }

        Ok(results)
    }

    fn matches_filters(&self, log_entry: &Value) -> bool {
        if let Some(level) = &self.level_filter {
            if let Some(log_level) = log_entry.get("level").and_then(|v| v.as_str()) {
                if log_level != level {
                    return false;
                }
            } else {
                return false;
            }
        }

        for (key, expected_value) in &self.field_filters {
            if let Some(actual_value) = log_entry.get(key).and_then(|v| v.as_str()) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

pub fn extract_timestamps(logs: &[Value]) -> Vec<String> {
    logs.iter()
        .filter_map(|log| log.get("timestamp").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .collect()
}