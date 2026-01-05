
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
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

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line_content) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                    continue;
                }
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect();
        
        Ok(filtered)
    }

    pub fn count_entries(&self) -> Result<usize, ParseError> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

pub fn analyze_logs(file_path: &str) -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new(file_path);
    
    println!("Analyzing logs from: {}", file_path);
    
    let total_entries = parser.count_entries()?;
    println!("Total log entries: {}", total_entries);
    
    let error_logs = parser.filter_by_level("error")?;
    println!("Error entries: {}", error_logs.len());
    
    let warning_logs = parser.filter_by_level("warning")?;
    println!("Warning entries: {}", warning_logs.len());
    
    if !error_logs.is_empty() {
        println!("\nRecent errors:");
        for entry in error_logs.iter().take(5) {
            println!("[{}] {}: {}", entry.timestamp, entry.service, entry.message);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "service": "api", "message": "Server started"}
{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "service": "database", "message": "Connection failed", "metadata": {"retry_count": 3}}
{"timestamp": "2024-01-15T10:32:00Z", "level": "WARNING", "service": "cache", "message": "Memory usage high"}"#;
        
        write!(file, "{}", log_data).unwrap();
        file
    }

    #[test]
    fn test_parse_logs() {
        let file = create_test_log();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();
        
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].service, "database");
    }

    #[test]
    fn test_filter_by_level() {
        let file = create_test_log();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let errors = parser.filter_by_level("error").unwrap();
        
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }

    #[test]
    fn test_count_entries() {
        let file = create_test_log();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let count = parser.count_entries().unwrap();
        
        assert_eq!(count, 3);
    }
}use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct LogFilter {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogFilter {
    pub fn new() -> Self {
        LogFilter {
            min_level: None,
            start_time: None,
            end_time: None,
        }
    }

    pub fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_string());
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(ref min_level) = self.min_level {
            if !self.check_level(&entry.level, min_level) {
                return false;
            }
        }

        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            if let Ok(entry_time) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                let utc_time = entry_time.with_timezone(&Utc);
                if utc_time < start || utc_time > end {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn check_level(&self, entry_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error"];
        let entry_idx = levels.iter().position(|&l| l == entry_level.to_lowercase());
        let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());

        match (entry_idx, min_idx) {
            (Some(e), Some(m)) => e >= m,
            _ => false,
        }
    }
}

pub struct LogParser {
    filter: LogFilter,
}

impl LogParser {
    pub fn new(filter: LogFilter) -> Self {
        LogParser { filter }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.filter.matches(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_filter_matches() {
        let filter = LogFilter::new()
            .with_min_level("info")
            .with_time_range(
                Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
            );

        let entry = LogEntry {
            timestamp: "2024-06-15T10:30:00Z".to_string(),
            level: "INFO".to_string(),
            message: "Test message".to_string(),
            extra: HashMap::new(),
        };

        assert!(filter.matches(&entry));
    }

    #[test]
    fn test_level_check() {
        let filter = LogFilter::new();
        assert!(filter.check_level("ERROR", "INFO"));
        assert!(!filter.check_level("DEBUG", "INFO"));
    }
}
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_json_log_file(path: &str) -> Result<Vec<LogEntry>, LogParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line_content = line?;
        
        if line_content.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line_content)?;
        
        let entry = parse_log_entry(json_value)
            .map_err(|e| LogParseError::MissingField(format!("Line {}: {}", line_num + 1, e)))?;
        
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_log_entry(value: Value) -> Result<LogEntry, String> {
    let obj = value.as_object().ok_or("Expected JSON object")?;
    
    let timestamp = obj.get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'timestamp' field")?
        .to_string();
    
    let level = obj.get("level")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'level' field")?
        .to_string();
    
    let message = obj.get("message")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'message' field")?
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