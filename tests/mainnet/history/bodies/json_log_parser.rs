
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

struct LogParser {
    entries: Vec<LogEntry>,
    stats: HashMap<String, usize>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            entries: Vec::new(),
            stats: HashMap::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    fn analyze(&mut self) {
        self.stats.clear();
        for entry in &self.entries {
            *self.stats.entry(entry.level.clone()).or_insert(0) += 1;
            *self.stats.entry(entry.service.clone()).or_insert(0) += 1;
        }
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    fn get_summary(&self) -> HashMap<String, usize> {
        self.stats.clone()
    }

    fn export_filtered<P: AsRef<Path>>(&self, path: P, entries: Vec<&LogEntry>) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = serde_json::ser::Serializer::new(file);

        let seq = serde_json::ser::Compound::Map(&mut writer);
        for entry in entries {
            serde_json::Serialize::serialize(entry, seq)?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    
    parser.load_from_file("logs.jsonl")?;
    parser.analyze();

    println!("Log Analysis Summary:");
    for (key, value) in parser.get_summary() {
        println!("{}: {}", key, value);
    }

    let error_logs = parser.filter_by_level("error");
    println!("\nFound {} error logs", error_logs.len());

    if !error_logs.is_empty() {
        parser.export_filtered("errors.json", error_logs)?;
        println!("Exported error logs to errors.json");
    }

    Ok(())
}use serde_json::Value;
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

pub struct LogParser {
    pub entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser { entries: Vec::new() }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(parsed) = serde_json::from_str::<Value>(&line) {
                let entry = LogEntry {
                    timestamp: parsed["timestamp"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    level: parsed["level"]
                        .as_str()
                        .unwrap_or("INFO")
                        .to_string(),
                    message: parsed["message"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    fields: parsed
                        .as_object()
                        .map(|obj| {
                            obj.iter()
                                .filter(|(k, _)| !["timestamp", "level", "message"].contains(k))
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect()
                        })
                        .unwrap_or_default(),
                };
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn search_in_message(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }

    pub fn get_field_values(&self, field_name: &str) -> Vec<&Value> {
        self.entries
            .iter()
            .filter_map(|entry| entry.fields.get(field_name))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","service":"api","request_id":"abc123"}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","message":"User login successful","user_id":42,"ip":"192.168.1.1"}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let mut parser = LogParser::new();
        parser.load_from_file(temp_file.path()).unwrap();

        assert_eq!(parser.entries.len(), 2);
        assert_eq!(parser.filter_by_level("ERROR").len(), 1);
        assert_eq!(parser.filter_by_level("INFO").len(), 1);
        
        let counts = parser.count_by_level();
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("INFO"), Some(&1));
        
        let error_entries = parser.search_in_message("failed");
        assert_eq!(error_entries.len(), 1);
        assert_eq!(error_entries[0].level, "ERROR");
        
        let request_ids = parser.get_field_values("request_id");
        assert_eq!(request_ids.len(), 1);
        assert_eq!(request_ids[0].as_str().unwrap(), "abc123");
    }
}
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
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

    pub fn extract_field(&self, field_name: &str) -> Result<Vec<String>, LogParseError> {
        let logs = self.parse_logs()?;
        let mut results = Vec::new();

        for log in logs {
            if let Some(value) = log.get(field_name) {
                results.push(value.to_string());
            } else {
                return Err(LogParseError::MissingField(field_name.to_string()));
            }
        }

        Ok(results)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "message": "Test message"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "message": "Error occurred"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse_logs();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_extract_field() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "message": "Test"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.extract_field("message");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["\"Test\""]);
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "message": "Info message"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "message": "Error message"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.filter_by_level("ERROR");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug)]
struct LogParser {
    min_level: String,
    filter_text: Option<String>,
}

impl LogParser {
    fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            filter_text: None,
        }
    }

    fn with_filter(mut self, filter: &str) -> Self {
        self.filter_text = Some(filter.to_lowercase());
        self
    }

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
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
        let entry_level = entry.level.to_lowercase();
        
        let level_priority = |level: &str| match level {
            "error" => 3,
            "warn" => 2,
            "info" => 1,
            "debug" => 0,
            _ => 0,
        };

        if level_priority(&entry_level) < level_priority(&self.min_level) {
            return false;
        }

        if let Some(filter) = &self.filter_text {
            if !entry.message.to_lowercase().contains(filter) {
                return false;
            }
        }

        true
    }

    fn print_summary(&self, entries: &[LogEntry]) {
        let mut counts = std::collections::HashMap::new();
        
        for entry in entries {
            *counts.entry(&entry.level).or_insert(0) += 1;
        }

        println!("Parsed {} log entries:", entries.len());
        for (level, count) in counts {
            println!("  {}: {}", level, count);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("info")
        .with_filter("connection");

    let entries = parser.parse_file("logs/app.log")?;
    parser.print_summary(&entries);

    if let Some(first_entry) = entries.first() {
        println!("\nFirst matching entry:");
        println!("Timestamp: {}", first_entry.timestamp);
        println!("Level: {}", first_entry.level);
        println!("Message: {}", first_entry.message);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_parser_filtering() {
        let test_entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            level: "ERROR".to_string(),
            message: "Database connection failed".to_string(),
            metadata: serde_json::json!({"service": "database"}),
        };

        let parser = LogParser::new("warn");
        assert!(parser.should_include(&test_entry));

        let parser = LogParser::new("error");
        assert!(parser.should_include(&test_entry));

        let parser = LogParser::new("error").with_filter("connection");
        assert!(parser.should_include(&test_entry));

        let parser = LogParser::new("error").with_filter("success");
        assert!(!parser.should_include(&test_entry));
    }
}