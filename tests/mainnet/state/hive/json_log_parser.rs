use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: Option<String>,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    filter_level: Option<String>,
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filter_level: None,
            required_fields: Vec::new(),
        }
    }

    pub fn set_level_filter(&mut self, level: &str) -> &mut Self {
        self.filter_level = Some(level.to_lowercase());
        self
    }

    pub fn add_required_field(&mut self, field: &str) -> &mut Self {
        self.required_fields.push(field.to_string());
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let json_value: Value = serde_json::from_str(line).ok()?;
        
        let mut fields = HashMap::new();
        let mut timestamp = None;
        let mut level = None;
        let mut message = None;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match key.as_str() {
                    "timestamp" | "time" | "@timestamp" => {
                        timestamp = value.as_str().map(|s| s.to_string());
                    }
                    "level" | "log_level" | "severity" => {
                        level = value.as_str().map(|s| s.to_lowercase());
                    }
                    "message" | "msg" | "log" => {
                        message = value.as_str().map(|s| s.to_string());
                    }
                    _ => {
                        fields.insert(key, value);
                    }
                }
            }
        }

        if let Some(filter) = &self.filter_level {
            if level.as_deref() != Some(filter) {
                return None;
            }
        }

        if !self.required_fields.is_empty() {
            for field in &self.required_fields {
                if !fields.contains_key(field) {
                    return None;
                }
            }
        }

        Some(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn extract_field_values(&self, entries: &[LogEntry], field_name: &str) -> Vec<Value> {
        entries
            .iter()
            .filter_map(|entry| entry.fields.get(field_name).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_with_filter() {
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "message": "Failed to connect", "service": "api"}"#;
        
        let mut parser = LogParser::new();
        parser.set_level_filter("error");
        
        let entry = parser.parse_line(log_data).unwrap();
        assert_eq!(entry.level, Some("error".to_string()));
        assert_eq!(entry.message, Some("Failed to connect".to_string()));
        assert_eq!(entry.fields.get("service").and_then(|v| v.as_str()), Some("api"));
    }

    #[test]
    fn test_field_extraction() {
        let entries = vec![
            LogEntry {
                timestamp: Some("2024-01-15T10:30:00Z".to_string()),
                level: Some("info".to_string()),
                message: Some("Request processed".to_string()),
                fields: vec![("user_id".to_string(), Value::String("user123".to_string()))]
                    .into_iter()
                    .collect(),
            },
        ];
        
        let parser = LogParser::new();
        let user_ids = parser.extract_field_values(&entries, "user_id");
        
        assert_eq!(user_ids.len(), 1);
        assert_eq!(user_ids[0].as_str(), Some("user123"));
    }
}
use serde_json::{Value, Error as JsonError};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum LogParseError {
    IoError(io::Error),
    JsonError(JsonError),
    InvalidLogFormat,
}

impl From<io::Error> for LogParseError {
    fn from(err: io::Error) -> Self {
        LogParseError::IoError(err)
    }
}

impl From<JsonError> for LogParseError {
    fn from(err: JsonError) -> Self {
        LogParseError::JsonError(err)
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub struct LogParser {
    min_level: Option<String>,
    filter_key: Option<String>,
    filter_value: Option<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            filter_key: None,
            filter_value: None,
        }
    }

    pub fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    pub fn with_filter(mut self, key: &str, value: &str) -> Self {
        self.filter_key = Some(key.to_string());
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, LogParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            match self.parse_line(&line) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => continue,
                Err(e) => eprintln!("Warning: Failed to parse line {}: {:?}", line_num + 1, e),
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<Option<LogEntry>, LogParseError> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp = json_value["timestamp"]
            .as_str()
            .ok_or(LogParseError::InvalidLogFormat)?
            .to_string();
            
        let level = json_value["level"]
            .as_str()
            .ok_or(LogParseError::InvalidLogFormat)?
            .to_string()
            .to_lowercase();

        if let Some(min_level) = &self.min_level {
            if !self.is_level_allowed(&level, min_level) {
                return Ok(None);
            }
        }

        if let (Some(key), Some(value)) = (&self.filter_key, &self.filter_value) {
            if let Some(metadata_value) = json_value.get(key) {
                if metadata_value.as_str() != Some(value) {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        }

        let message = json_value["message"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let metadata = json_value["metadata"].clone();

        Ok(Some(LogEntry {
            timestamp,
            level,
            message,
            metadata,
        }))
    }

    fn is_level_allowed(&self, log_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error", "fatal"];
        
        let log_idx = levels.iter().position(|&l| l == log_level);
        let min_idx = levels.iter().position(|&l| l == min_level);
        
        match (log_idx, min_idx) {
            (Some(l), Some(m)) => l >= m,
            _ => false,
        }
    }
}

pub fn print_log_summary(entries: &[LogEntry]) {
    let mut level_counts = std::collections::HashMap::new();
    
    for entry in entries {
        *level_counts.entry(entry.level.clone()).or_insert(0) += 1;
    }
    
    println!("Log Summary:");
    println!("Total entries: {}", entries.len());
    
    let levels = ["error", "warn", "info", "debug", "trace"];
    for level in levels.iter() {
        if let Some(count) = level_counts.get(*level) {
            println!("  {}: {}", level, count);
        }
    }
    
    if let Some(first) = entries.first() {
        println!("Time range: {} to {}", 
                 first.timestamp, 
                 entries.last().map(|e| &e.timestamp).unwrap_or(&first.timestamp));
    }
}use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: Option<String>,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    pub entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                self.entries.push(entry);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let mut entry = LogEntry {
            timestamp: json_value.get("timestamp").and_then(|v| v.as_str()).map(String::from),
            level: json_value.get("level").and_then(|v| v.as_str()).map(String::from),
            message: json_value.get("message").and_then(|v| v.as_str()).map(String::from),
            fields: HashMap::new(),
        };

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match key.as_str() {
                    "timestamp" | "level" | "message" => continue,
                    _ => {
                        entry.fields.insert(key, value.clone());
                    }
                }
            }
        }

        Ok(entry)
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.as_deref() == Some(level))
            .collect()
    }

    pub fn extract_field(&self, field_name: &str) -> Vec<Option<&Value>> {
        self.entries
            .iter()
            .map(|entry| entry.fields.get(field_name))
            .collect()
    }

    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        for entry in &self.entries {
            if let Some(level) = &entry.level {
                *stats.entry(level.clone()).or_insert(0) += 1;
            }
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_line() {
        let parser = LogParser::new();
        let json_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","user_id":12345,"action":"login"}"#;
        
        let entry = parser.parse_line(json_line).unwrap();
        
        assert_eq!(entry.timestamp, Some("2024-01-15T10:30:00Z".to_string()));
        assert_eq!(entry.level, Some("INFO".to_string()));
        assert_eq!(entry.message, Some("System started".to_string()));
        assert_eq!(entry.fields.get("user_id"), Some(&json!(12345)));
        assert_eq!(entry.fields.get("action"), Some(&json!("login")));
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        
        let entry1 = LogEntry {
            timestamp: Some("2024-01-15T10:30:00Z".to_string()),
            level: Some("INFO".to_string()),
            message: Some("System started".to_string()),
            fields: HashMap::new(),
        };
        
        let entry2 = LogEntry {
            timestamp: Some("2024-01-15T10:31:00Z".to_string()),
            level: Some("ERROR".to_string()),
            message: Some("Connection failed".to_string()),
            fields: HashMap::new(),
        };
        
        parser.entries.push(entry1);
        parser.entries.push(entry2);
        
        let errors = parser.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message.as_deref(), Some("Connection failed"));
    }
}use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    filters: HashMap<String, String>,
    format_options: FormatOptions,
}

pub struct FormatOptions {
    pub show_timestamp: bool,
    pub show_level: bool,
    pub show_component: bool,
    pub indent: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            show_timestamp: true,
            show_level: true,
            show_component: true,
            indent: 2,
        }
    }
}

impl LogParser {
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
            format_options: FormatOptions::default(),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.matches_filters(&json_value) {
                    logs.push(json_value);
                }
            }
        }

        Ok(logs)
    }

    fn matches_filters(&self, json_value: &Value) -> bool {
        for (key, expected_value) in &self.filters {
            if let Some(actual_value) = json_value.get(key) {
                if actual_value.as_str() != Some(expected_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn format_log(&self, log: &Value) -> String {
        let mut output = String::new();

        if self.format_options.show_timestamp {
            if let Some(timestamp) = log.get("timestamp").and_then(|v| v.as_str()) {
                output.push_str(&format!("[{}] ", timestamp));
            }
        }

        if self.format_options.show_level {
            if let Some(level) = log.get("level").and_then(|v| v.as_str()) {
                output.push_str(&format!("{}: ", level.to_uppercase()));
            }
        }

        if self.format_options.show_component {
            if let Some(component) = log.get("component").and_then(|v| v.as_str()) {
                output.push_str(&format!("{{{}}} ", component));
            }
        }

        if let Some(message) = log.get("message").and_then(|v| v.as_str()) {
            output.push_str(message);
        }

        if let Some(data) = log.get("data") {
            if !data.is_null() {
                let formatted_data = serde_json::to_string_pretty(data).unwrap_or_default();
                let indented_data = formatted_data
                    .lines()
                    .map(|line| format!("{:width$}{}", "", line, width = self.format_options.indent))
                    .collect::<Vec<String>>()
                    .join("\n");
                output.push_str(&format!("\n{}", indented_data));
            }
        }

        output
    }
}

pub fn analyze_log_levels(logs: &[Value]) -> HashMap<String, usize> {
    let mut level_counts = HashMap::new();
    
    for log in logs {
        if let Some(level) = log.get("level").and_then(|v| v.as_str()) {
            *level_counts.entry(level.to_string()).or_insert(0) += 1;
        }
    }
    
    level_counts
}use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

struct LogParser {
    min_level: String,
    service_filter: Option<String>,
}

impl LogParser {
    fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            service_filter: None,
        }
    }

    fn with_service_filter(mut self, service: &str) -> Self {
        self.service_filter = Some(service.to_string());
        self
    }

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(mut entry) => {
                    entry.level = entry.level.to_lowercase();
                    if self.should_include(&entry) {
                        entries.push(entry);
                    }
                }
                Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
            }
        }

        Ok(entries)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        if !self.is_level_sufficient(&entry.level) {
            return false;
        }

        if let Some(ref service) = self.service_filter {
            if entry.service != *service {
                return false;
            }
        }

        true
    }

    fn is_level_sufficient(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error"];
        let min_index = level_order.iter().position(|&l| l == self.min_level);
        let entry_index = level_order.iter().position(|&l| l == level);

        match (min_index, entry_index) {
            (Some(min), Some(entry)) => entry >= min,
            _ => false,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("info")
        .with_service_filter("api-service");

    let entries = parser.parse_file("logs/app.log")?;
    
    println!("Found {} log entries", entries.len());
    for entry in entries.iter().take(5) {
        println!("{:?}", entry);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parser_filters_by_level() {
        let log_data = r#"
{"timestamp": "2023-01-01T00:00:00Z", "level": "DEBUG", "service": "test", "message": "debug message"}
{"timestamp": "2023-01-01T00:00:01Z", "level": "INFO", "service": "test", "message": "info message"}
{"timestamp": "2023-01-01T00:00:02Z", "level": "ERROR", "service": "test", "message": "error message"}
"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", log_data).unwrap();

        let parser = LogParser::new("info");
        let entries = parser.parse_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.level != "debug"));
    }

    #[test]
    fn test_parser_filters_by_service() {
        let log_data = r#"
{"timestamp": "2023-01-01T00:00:00Z", "level": "INFO", "service": "api", "message": "api message"}
{"timestamp": "2023-01-01T00:00:01Z", "level": "INFO", "service": "db", "message": "db message"}
"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", log_data).unwrap();

        let parser = LogParser::new("info").with_service_filter("api");
        let entries = parser.parse_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].service, "api");
    }
}