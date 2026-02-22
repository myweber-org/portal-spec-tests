use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogFilter {
    min_level: String,
    target_service: Option<String>,
}

impl LogFilter {
    pub fn new(min_level: &str, service: Option<&str>) -> Self {
        LogFilter {
            min_level: min_level.to_lowercase(),
            target_service: service.map(|s| s.to_string()),
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut filtered_logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(parsed) = serde_json::from_str::<Value>(&line) {
                if self.should_include(&parsed) {
                    filtered_logs.push(line);
                }
            }
        }

        Ok(filtered_logs)
    }

    fn should_include(&self, log_entry: &Value) -> bool {
        let level = log_entry.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

        let level_priority = self.get_level_priority(&level);
        let min_priority = self.get_level_priority(&self.min_level);

        if level_priority < min_priority {
            return false;
        }

        if let Some(ref target_service) = self.target_service {
            let service = log_entry.get("service")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if service != target_service {
                return false;
            }
        }

        true
    }

    fn get_level_priority(&self, level: &str) -> u8 {
        match level {
            "error" => 1,
            "warn" => 2,
            "info" => 3,
            "debug" => 4,
            "trace" => 5,
            _ => 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_by_level() {
        let logs = r#"{"level": "ERROR", "message": "System failure", "service": "api"}
{"level": "INFO", "message": "User login", "service": "auth"}
{"level": "DEBUG", "message": "Processing request", "service": "api"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", logs).unwrap();

        let filter = LogFilter::new("info", None);
        let result = filter.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result[0].contains("System failure"));
        assert!(result[1].contains("User login"));
    }

    #[test]
    fn test_filter_by_service() {
        let logs = r#"{"level": "ERROR", "message": "DB error", "service": "database"}
{"level": "WARN", "message": "High latency", "service": "api"}
{"level": "INFO", "message": "Cache miss", "service": "database"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", logs).unwrap();

        let filter = LogFilter::new("warn", Some("database"));
        let result = filter.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].contains("DB error"));
    }
}use serde_json::{Value, Error as JsonError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
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
        let line = line?;
        match parse_log_line(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
        }
    }

    Ok(entries)
}

fn parse_log_line(line: &str) -> Result<LogEntry, LogError> {
    let json_value: Value = serde_json::from_str(line)?;

    let obj = json_value.as_object()
        .ok_or_else(|| LogError::InvalidFormat("Expected JSON object".to_string()))?;

    let timestamp = obj.get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogError::InvalidFormat("Missing timestamp field".to_string()))?
        .to_string();

    let level = obj.get("level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogError::InvalidFormat("Missing level field".to_string()))?
        .to_string();

    let message = obj.get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogError::InvalidFormat("Missing message field".to_string()))?
        .to_string();

    let mut fields = HashMap::new();
    for (key, value) in obj {
        if !["timestamp", "level", "message"].contains(&key.as_str()) {
            fields.insert(key.clone(), value.clone());
        }
    }

    Ok(LogEntry {
        timestamp,
        level,
        message,
        fields,
    })
}

pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level.eq_ignore_ascii_case(level))
        .collect()
}

pub fn count_by_level(entries: &[LogEntry]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for entry in entries {
        *counts.entry(entry.level.clone()).or_insert(0) += 1;
    }
    counts
}