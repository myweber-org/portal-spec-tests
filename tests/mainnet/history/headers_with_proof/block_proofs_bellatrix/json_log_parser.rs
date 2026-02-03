use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
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

pub struct JsonLogParser {
    file_path: String,
}

impl JsonLogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse_logs(&self) -> Result<Vec<Value>, LogParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }

            let json_value: Value = serde_json::from_str(&line_content)?;
            logs.push(json_value);
        }

        Ok(logs)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<Value>, LogParseError> {
        let logs = self.parse_logs()?;
        let filtered: Vec<Value> = logs
            .into_iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|l| l.eq_ignore_ascii_case(level))
                    .unwrap_or(false)
            })
            .collect();

        Ok(filtered)
    }

    pub fn extract_timestamps(&self) -> Result<Vec<String>, LogParseError> {
        let logs = self.parse_logs()?;
        let mut timestamps = Vec::new();

        for log in logs {
            if let Some(timestamp) = log.get("timestamp").and_then(|v| v.as_str()) {
                timestamps.push(timestamp.to_string());
            }
        }

        Ok(timestamps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "timestamp": "2023-10-01T12:00:00Z", "message": "System started"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "timestamp": "2023-10-01T12:05:00Z", "message": "Connection failed"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let logs = parser.parse_logs().unwrap();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "timestamp": "2023-10-01T12:00:00Z", "message": "Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "timestamp": "2023-10-01T12:05:00Z", "message": "Error"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["level"], "ERROR");
    }
}use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    module: String,
    thread_id: u32,
}

struct LogFilter {
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    module_filter: Option<String>,
}

impl LogFilter {
    fn new(min_level: LogLevel) -> Self {
        LogFilter {
            min_level,
            start_time: None,
            end_time: None,
            module_filter: None,
        }
    }

    fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    fn with_module_filter(mut self, module: &str) -> Self {
        self.module_filter = Some(module.to_string());
        self
    }

    fn matches(&self, entry: &LogEntry) -> bool {
        if entry.level > self.min_level {
            return false;
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        if let Some(ref module_filter) = self.module_filter {
            if !entry.module.contains(module_filter) {
                return false;
            }
        }

        true
    }
}

fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let entry: LogEntry = serde_json::from_str(&line)?;
        entries.push(entry);
    }

    Ok(entries)
}

fn filter_logs(entries: Vec<LogEntry>, filter: &LogFilter) -> Vec<LogEntry> {
    entries.into_iter()
        .filter(|entry| filter.matches(entry))
        .collect()
}

fn print_log_summary(entries: &[LogEntry]) {
    let error_count = entries.iter().filter(|e| e.level == LogLevel::ERROR).count();
    let warn_count = entries.iter().filter(|e| e.level == LogLevel::WARN).count();
    let info_count = entries.iter().filter(|e| e.level == LogLevel::INFO).count();

    println!("Log Summary:");
    println!("  Total entries: {}", entries.len());
    println!("  Errors: {}", error_count);
    println!("  Warnings: {}", warn_count);
    println!("  Info messages: {}", info_count);

    if !entries.is_empty() {
        let first = entries.first().unwrap().timestamp;
        let last = entries.last().unwrap().timestamp;
        let duration = last - first;
        println!("  Time range: {} to {}", first, last);
        println!("  Duration: {} seconds", duration.num_seconds());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entries = parse_log_file("application.log")?;
    
    let filter = LogFilter::new(LogLevel::INFO)
        .with_module_filter("database");
    
    let filtered_entries = filter_logs(entries, &filter);
    
    print_log_summary(&filtered_entries);
    
    for entry in filtered_entries.iter().take(5) {
        println!("{:?}", entry);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_filter() {
        let entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            level: LogLevel::ERROR,
            message: "Test error".to_string(),
            module: "database".to_string(),
            thread_id: 1,
        };

        let filter = LogFilter::new(LogLevel::WARN);
        assert!(!filter.matches(&entry));

        let filter = LogFilter::new(LogLevel::ERROR);
        assert!(filter.matches(&entry));
    }

    #[test]
    fn test_time_filter() {
        let entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
            level: LogLevel::INFO,
            message: "Test".to_string(),
            module: "app".to_string(),
            thread_id: 1,
        };

        let start = Utc.with_ymd_and_hms(2024, 1, 15, 11, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 15, 13, 0, 0).unwrap();

        let filter = LogFilter::new(LogLevel::INFO)
            .with_time_range(start, end);
        
        assert!(filter.matches(&entry));
    }
}