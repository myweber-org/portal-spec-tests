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
    filter_level: Option<String>,
    include_fields: Vec<String>,
    exclude_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filter_level: None,
            include_fields: Vec::new(),
            exclude_fields: Vec::new(),
        }
    }

    pub fn set_level_filter(&mut self, level: &str) -> &mut Self {
        self.filter_level = Some(level.to_lowercase());
        self
    }

    pub fn include_field(&mut self, field: &str) -> &mut Self {
        self.include_fields.push(field.to_string());
        self
    }

    pub fn exclude_field(&mut self, field: &str) -> &mut Self {
        self.exclude_fields.push(field.to_string());
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

        if let Some(filter) = &self.filter_level {
            if &level != filter {
                return Err("Level filtered out".into());
            }
        }

        let message = json_value.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key == "timestamp" || key == "level" || key == "message" {
                    continue;
                }

                if !self.include_fields.is_empty() && !self.include_fields.contains(key) {
                    continue;
                }

                if self.exclude_fields.contains(key) {
                    continue;
                }

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

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!("[{}] {}: {}", 
            entry.timestamp, 
            entry.level.to_uppercase(), 
            entry.message
        );

        if !entry.fields.is_empty() {
            output.push_str(" {");
            let field_strings: Vec<String> = entry.fields.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            output.push_str(&field_strings.join(", "));
            output.push('}');
        }

        output
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let parser = LogParser::new();
        let json_line = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "error", "message": "Failed to connect", "service": "api", "attempt": 3}"#;
        
        let result = parser.parse_line(json_line);
        assert!(result.is_ok());
        
        let entry = result.unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "error");
        assert_eq!(entry.message, "Failed to connect");
        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields.get("service").unwrap().as_str().unwrap(), "api");
    }

    #[test]
    fn test_level_filter() {
        let mut parser = LogParser::new();
        parser.set_level_filter("error");
        
        let error_line = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "error", "message": "Error"}"#;
        let info_line = r#"{"timestamp": "2024-01-15T10:31:00Z", "level": "info", "message": "Info"}"#;
        
        assert!(parser.parse_line(error_line).is_ok());
        assert!(parser.parse_line(info_line).is_err());
    }

    #[test]
    fn test_format_entry() {
        let parser = LogParser::new();
        let mut fields = HashMap::new();
        fields.insert("service".to_string(), Value::String("api".to_string()));
        fields.insert("code".to_string(), Value::Number(500.into()));
        
        let entry = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: "error".to_string(),
            message: "Failed to connect".to_string(),
            fields,
        };
        
        let formatted = parser.format_entry(&entry);
        assert!(formatted.contains("[2024-01-15T10:30:00Z]"));
        assert!(formatted.contains("ERROR: Failed to connect"));
        assert!(formatted.contains("service: \"api\""));
        assert!(formatted.contains("code: 500"));
    }
}
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
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
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub metadata: serde_json::Value,
}

pub struct LogParser {
    file_path: String,
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
            min_level: LogLevel::INFO,
            start_time: None,
            end_time: None,
        }
    }

    pub fn set_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    pub fn set_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;

            if !self.filter_entry(&entry) {
                continue;
            }

            entries.push(entry);
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

    pub fn count_by_level(&self) -> Result<std::collections::HashMap<LogLevel, usize>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let mut counts = std::collections::HashMap::new();

        for entry in entries {
            *counts.entry(entry.level).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_val = match self {
            LogLevel::ERROR => 4,
            LogLevel::WARN => 3,
            LogLevel::INFO => 2,
            LogLevel::DEBUG => 1,
            LogLevel::TRACE => 0,
        };
        let other_val = match other {
            LogLevel::ERROR => 4,
            LogLevel::WARN => 3,
            LogLevel::INFO => 2,
            LogLevel::DEBUG => 1,
            LogLevel::TRACE => 0,
        };
        Some(self_val.cmp(&other_val))
    }
}