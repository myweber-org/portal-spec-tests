use serde_json::{Value, Error as JsonError};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(JsonError),
    InvalidStructure(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<JsonError> for LogError {
    fn from(err: JsonError) -> Self {
        LogError::ParseError(err)
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_json_log<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, LogError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line_content = line?;
        
        if line_content.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line_content)?;
        
        let entry = extract_log_entry(json_value)
            .map_err(|msg| LogError::InvalidStructure(format!("Line {}: {}", line_num + 1, msg)))?;
        
        entries.push(entry);
    }

    Ok(entries)
}

fn extract_log_entry(value: Value) -> Result<LogEntry, String> {
    let obj = value.as_object()
        .ok_or_else(|| "Log entry must be a JSON object".to_string())?;

    let timestamp = obj.get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid timestamp field".to_string())?
        .to_string();

    let level = obj.get("level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid level field".to_string())?
        .to_string();

    let message = obj.get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid message field".to_string())?
        .to_string();

    let metadata = obj.get("metadata")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    Ok(LogEntry {
        timestamp,
        level,
        message,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2023-10-01T12:00:00Z", "level": "INFO", "message": "System started", "metadata": {{"user": "admin"}}}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2023-10-01T12:01:00Z", "level": "ERROR", "message": "Connection failed"}}"#).unwrap();

        let entries = parse_json_log(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].message, "Connection failed");
    }

    #[test]
    fn test_parse_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{invalid json}}"#).unwrap();

        let result = parse_json_log(temp_file.path());
        assert!(matches!(result, Err(LogError::ParseError(_))));
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    service: String,
    message: String,
    metadata: HashMap<String, String>,
}

struct LogProcessor {
    min_level: String,
    service_filter: Option<String>,
}

impl LogProcessor {
    fn new(min_level: &str) -> Self {
        LogProcessor {
            min_level: min_level.to_lowercase(),
            service_filter: None,
        }
    }

    fn with_service_filter(mut self, service: &str) -> Self {
        self.service_filter = Some(service.to_string());
        self
    }

    fn process_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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
        let level_priority = |level: &str| match level.to_lowercase().as_str() {
            "error" => 3,
            "warn" => 2,
            "info" => 1,
            "debug" => 0,
            _ => 0,
        };

        let entry_priority = level_priority(&entry.level);
        let min_priority = level_priority(&self.min_level);

        if entry_priority < min_priority {
            return false;
        }

        if let Some(ref service) = self.service_filter {
            if entry.service != *service {
                return false;
            }
        }

        true
    }

    fn generate_summary(&self, entries: &[LogEntry]) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        for entry in entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }

        summary
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = LogProcessor::new("info")
        .with_service_filter("api-service");

    let entries = processor.process_file("logs/app.log")?;
    
    println!("Found {} log entries", entries.len());
    
    let summary = processor.generate_summary(&entries);
    for (level, count) in summary {
        println!("{}: {}", level, count);
    }

    if let Some(error_entry) = entries.iter().find(|e| e.level == "error") {
        println!("Latest error: {} - {}", error_entry.timestamp, error_entry.message);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_filtering() {
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api-service","message":"Request processed","metadata":{"method":"GET","path":"/api/users"}}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"auth-service","message":"Authentication failed","metadata":{"user_id":"123"}}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"api-service","message":"Slow response","metadata":{"duration_ms":"1500"}}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let processor = LogProcessor::new("warn")
            .with_service_filter("api-service");

        let entries = processor.process_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "WARN");
        assert_eq!(entries[0].service, "api-service");
    }
}