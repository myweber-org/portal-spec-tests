use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    filters: HashMap<String, String>,
    format_template: String,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: HashMap::new(),
            format_template: String::from("{timestamp} - {level} - {message}"),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn set_format(&mut self, format: &str) {
        self.format_template = format.to_string();
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.matches_filters(&json_value) {
                    if let Some(formatted) = self.format_entry(&json_value) {
                        results.push(formatted);
                    }
                }
            }
        }

        Ok(results)
    }

    fn matches_filters(&self, json: &Value) -> bool {
        for (key, expected_value) in &self.filters {
            if let Some(actual_value) = json.get(key) {
                if actual_value.as_str() != Some(expected_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn format_entry(&self, json: &Value) -> Option<String> {
        let mut result = self.format_template.clone();
        
        for (key, value) in json.as_object()? {
            let placeholder = format!("{{{}}}", key);
            if let Some(str_value) = value.as_str() {
                result = result.replace(&placeholder, str_value);
            } else {
                result = result.replace(&placeholder, &value.to_string());
            }
        }
        
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut parser = LogParser::new();
        parser.add_filter("level", "ERROR");
        parser.set_format("{timestamp} :: {level} :: {message}");

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2023-10-01T12:00:00Z", "level": "ERROR", "message": "Database connection failed"}}"#
        ).unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2023-10-01T12:01:00Z", "level": "INFO", "message": "User logged in"}}"#
        ).unwrap();

        let results = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "2023-10-01T12:00:00Z :: ERROR :: Database connection failed");
    }
}use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
}

pub struct LogParser {
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogParser {
    pub fn new(min_level: LogLevel) -> Self {
        LogParser {
            min_level,
            start_time: None,
            end_time: None,
        }
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
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

            let entry: LogEntry = serde_json::from_str(&line)?;
            
            if self.filter_entry(&entry) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn filter_entry(&self, entry: &LogEntry) -> bool {
        if entry.level < self.min_level {
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

        true
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_val = match self {
            LogLevel::DEBUG => 0,
            LogLevel::INFO => 1,
            LogLevel::WARN => 2,
            LogLevel::ERROR => 3,
        };
        let other_val = match other {
            LogLevel::DEBUG => 0,
            LogLevel::INFO => 1,
            LogLevel::WARN => 2,
            LogLevel::ERROR => 3,
        };
        Some(self_val.cmp(&other_val))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_parser_filtering() {
        let parser = LogParser::new(LogLevel::WARN)
            .with_time_range(
                Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
            );

        let test_entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 6, 15, 10, 30, 0).unwrap(),
            level: LogLevel::ERROR,
            message: "Test error".to_string(),
            component: "test".to_string(),
        };

        assert!(parser.filter_entry(&test_entry));
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::DEBUG < LogLevel::INFO);
        assert!(LogLevel::INFO < LogLevel::WARN);
        assert!(LogLevel::WARN < LogLevel::ERROR);
    }
}use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
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

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line_content) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line_content, e),
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect();

        Ok(filtered)
    }

    pub fn count_entries(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"auth","message":"Authentication failed","metadata":{"user_id":123}}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"api","message":"Request processed","metadata":null}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"database","message":"Slow query detected","metadata":{"query_time":2.5}}"#;
        
        write!(file, "{}", log_data).unwrap();
        file
    }

    #[test]
    fn test_parse_logs() {
        let test_file = create_test_log();
        let parser = LogParser::new(test_file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();
        
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, LogLevel::ERROR);
        assert_eq!(entries[1].level, LogLevel::INFO);
    }

    #[test]
    fn test_filter_by_level() {
        let test_file = create_test_log();
        let parser = LogParser::new(test_file.path().to_str().unwrap());
        let error_logs = parser.filter_by_level(LogLevel::ERROR).unwrap();
        
        assert_eq!(error_logs.len(), 1);
        assert_eq!(error_logs[0].service, "auth");
    }

    #[test]
    fn test_count_entries() {
        let test_file = create_test_log();
        let parser = LogParser::new(test_file.path().to_str().unwrap());
        let count = parser.count_entries().unwrap();
        
        assert_eq!(count, 3);
    }
}