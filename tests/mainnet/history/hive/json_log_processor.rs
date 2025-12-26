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
}