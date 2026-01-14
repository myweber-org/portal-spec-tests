use serde_json::{Value, Error as JsonError};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    JsonError(JsonError),
    InvalidFormat(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<JsonError> for LogError {
    fn from(err: JsonError) -> Self {
        LogError::JsonError(err)
    }
}

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, LogError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line_content = line?;
        
        match parse_log_line(&line_content) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
        }
    }

    Ok(entries)
}

fn parse_log_line(line: &str) -> Result<LogEntry, LogError> {
    let json_value: Value = serde_json::from_str(line)?;
    
    let timestamp = json_value["timestamp"]
        .as_str()
        .ok_or_else(|| LogError::InvalidFormat("Missing timestamp field".to_string()))?
        .to_string();
    
    let level = json_value["level"]
        .as_str()
        .ok_or_else(|| LogError::InvalidFormat("Missing level field".to_string()))?
        .to_string();
    
    let message = json_value["message"]
        .as_str()
        .ok_or_else(|| LogError::InvalidFormat("Missing message field".to_string()))?
        .to_string();
    
    let metadata = json_value.get("metadata")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    Ok(LogEntry {
        timestamp,
        level,
        message,
        metadata,
    })
}

pub fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
        .collect()
}

pub fn extract_timestamps(entries: &[LogEntry]) -> Vec<String> {
    entries.iter()
        .map(|entry| entry.timestamp.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_log_line() {
        let json_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","metadata":{"user":"admin"}}"#;
        let entry = parse_log_line(json_line).unwrap();
        
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "System started");
        assert_eq!(entry.metadata["user"], "admin");
    }

    #[test]
    fn test_filter_logs() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Test info".to_string(),
                metadata: json!({}),
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Test error".to_string(),
                metadata: json!({}),
            },
        ];
        
        let filtered = filter_logs_by_level(&entries, "ERROR");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, "ERROR");
    }
}use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogFilter {
    min_level: Option<String>,
    contains_text: Option<String>,
}

impl LogFilter {
    pub fn new(min_level: Option<String>, contains_text: Option<String>) -> Self {
        LogFilter {
            min_level,
            contains_text,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut filtered_logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(log_entry) = serde_json::from_str::<Value>(&line) {
                if self.matches_filter(&log_entry) {
                    filtered_logs.push(log_entry);
                }
            }
        }

        Ok(filtered_logs)
    }

    fn matches_filter(&self, log_entry: &Value) -> bool {
        if let Some(min_level) = &self.min_level {
            if let Some(level) = log_entry.get("level").and_then(|v| v.as_str()) {
                let level_order = self.level_order(level);
                let min_order = self.level_order(min_level);
                if level_order < min_order {
                    return false;
                }
            }
        }

        if let Some(text) = &self.contains_text {
            let log_string = log_entry.to_string();
            if !log_string.contains(text) {
                return false;
            }
        }

        true
    }

    fn level_order(&self, level: &str) -> u8 {
        match level.to_lowercase().as_str() {
            "debug" => 1,
            "info" => 2,
            "warn" => 3,
            "error" => 4,
            "critical" => 5,
            _ => 0,
        }
    }
}

pub fn print_log_summary(logs: &[Value]) {
    let mut level_counts = std::collections::HashMap::new();
    
    for log in logs {
        if let Some(level) = log.get("level").and_then(|v| v.as_str()) {
            *level_counts.entry(level.to_string()).or_insert(0) += 1;
        }
    }

    println!("Log Summary:");
    for (level, count) in level_counts {
        println!("  {}: {}", level, count);
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    ValidationError(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<serde_json::Error> for LogError {
    fn from(err: serde_json::Error) -> Self {
        LogError::ParseError(err)
    }
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::IoError(e) => write!(f, "IO error: {}", e),
            LogError::ParseError(e) => write!(f, "Parse error: {}", e),
            LogError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl Error for LogError {}

impl LogEntry {
    pub fn validate(&self) -> Result<(), LogError> {
        if self.timestamp.is_empty() {
            return Err(LogError::ValidationError("Timestamp cannot be empty".to_string()));
        }
        
        let valid_levels = ["INFO", "WARN", "ERROR", "DEBUG"];
        if !valid_levels.contains(&self.level.as_str()) {
            return Err(LogError::ValidationError(
                format!("Invalid log level: {}", self.level)
            ));
        }
        
        if self.service.is_empty() {
            return Err(LogError::ValidationError("Service name cannot be empty".to_string()));
        }
        
        Ok(())
    }
    
    pub fn is_error(&self) -> bool {
        self.level == "ERROR"
    }
    
    pub fn service_name(&self) -> &str {
        &self.service
    }
}

pub struct LogProcessor {
    entries: Vec<LogEntry>,
    error_count: usize,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            error_count: 0,
        }
    }
    
    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            match serde_json::from_str::<LogEntry>(&line) {
                Ok(mut entry) => {
                    if let Err(e) = entry.validate() {
                        eprintln!("Line {} validation failed: {}", line_num + 1, e);
                        self.error_count += 1;
                        continue;
                    }
                    self.entries.push(entry);
                }
                Err(e) => {
                    eprintln!("Line {} parse failed: {}", line_num + 1, e);
                    self.error_count += 1;
                }
            }
        }
        
        Ok(())
    }
    
    pub fn error_entries(&self) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| e.is_error()).collect()
    }
    
    pub fn entries_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|e| e.service_name() == service)
            .collect()
    }
    
    pub fn total_processed(&self) -> usize {
        self.entries.len()
    }
    
    pub fn error_count(&self) -> usize {
        self.error_count
    }
    
    pub fn export_json<P: AsRef<Path>>(&self, path: P) -> Result<(), LogError> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &self.entries)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_log_entry() {
        let json = r#"{
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "INFO",
            "service": "api-gateway",
            "message": "Request processed successfully",
            "metadata": {"request_id": "abc123"}
        }"#;
        
        let entry: LogEntry = serde_json::from_str(json).unwrap();
        assert!(entry.validate().is_ok());
        assert!(!entry.is_error());
        assert_eq!(entry.service_name(), "api-gateway");
    }
    
    #[test]
    fn test_invalid_level() {
        let json = r#"{
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "INVALID",
            "service": "api-gateway",
            "message": "Test message"
        }"#;
        
        let entry: LogEntry = serde_json::from_str(json).unwrap();
        assert!(entry.validate().is_err());
    }
    
    #[test]
    fn test_process_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let logs = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"service-a","message":"Test 1"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"service-b","message":"Test 2"}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"service-a","message":"Test 3"}"#;
        
        std::fs::write(temp_file.path(), logs).unwrap();
        
        let mut processor = LogProcessor::new();
        processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(processor.total_processed(), 3);
        assert_eq!(processor.error_entries().len(), 1);
        assert_eq!(processor.entries_by_service("service-a").len(), 2);
    }
}