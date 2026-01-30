
use serde_json::{Value, Error};
use chrono::{DateTime, Utc};

pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_json_log(json_str: &str) -> Result<LogEntry, String> {
    let parsed: Result<Value, Error> = serde_json::from_str(json_str);
    
    match parsed {
        Ok(data) => {
            let timestamp = match data.get("timestamp") {
                Some(ts) => {
                    let ts_str = ts.as_str().unwrap_or("");
                    DateTime::parse_from_rfc3339(ts_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now())
                }
                None => Utc::now(),
            };

            let level = data.get("level")
                .and_then(|l| l.as_str())
                .unwrap_or("INFO")
                .to_string();

            let message = data.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string();

            let metadata = data.clone();

            Ok(LogEntry {
                timestamp,
                level,
                message,
                metadata,
            })
        }
        Err(e) => Err(format!("Failed to parse JSON: {}", e)),
    }
}

pub fn filter_logs_by_level(logs: Vec<LogEntry>, min_level: &str) -> Vec<LogEntry> {
    let level_order = vec!["DEBUG", "INFO", "WARN", "ERROR", "FATAL"];
    
    let min_index = level_order.iter()
        .position(|&l| l == min_level)
        .unwrap_or(0);

    logs.into_iter()
        .filter(|log| {
            level_order.iter()
                .position(|&l| l == log.level.as_str())
                .map(|idx| idx >= min_index)
                .unwrap_or(false)
        })
        .collect()
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn from_str(level: &str) -> Option<Self> {
        match level.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warning" => Some(LogLevel::Warning),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            "trace" => Some(LogLevel::Trace),
            _ => None,
        }
    }

    fn severity(&self) -> u8 {
        match self {
            LogLevel::Error => 5,
            LogLevel::Warning => 4,
            LogLevel::Info => 3,
            LogLevel::Debug => 2,
            LogLevel::Trace => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    include_fields: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            include_fields: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    pub fn with_include_fields(mut self, fields: Vec<String>) -> Self {
        self.include_fields = fields;
        self
    }

    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;
            
            if let Some(entry) = self.parse_line(&line) {
                if self.should_include(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parsed: serde_json::Value = serde_json::from_str(line).ok()?;
        
        let timestamp = parsed.get("timestamp")?.as_str()?.to_string();
        let level_str = parsed.get("level")?.as_str()?;
        let level = LogLevel::from_str(level_str)?;
        let message = parsed.get("message")?.as_str()?.to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = parsed.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    if let Some(str_val) = value.as_str() {
                        fields.insert(key.clone(), str_val.to_string());
                    }
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

    fn should_include(&self, entry: &LogEntry) -> bool {
        if entry.level.severity() < self.min_level.severity() {
            return false;
        }

        for pattern in &self.exclude_patterns {
            if entry.message.contains(pattern) {
                return false;
            }
        }

        if !self.include_fields.is_empty() {
            for field in &self.include_fields {
                if !entry.fields.contains_key(field) {
                    return false;
                }
            }
        }

        true
    }

    pub fn summarize(&self, entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
        let mut summary = HashMap::new();
        for entry in entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let json_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Failed to connect","service":"api","user_id":"123"}"#;
        
        let processor = LogProcessor::new(LogLevel::Trace);
        let entry = processor.parse_line(json_line).unwrap();
        
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.message, "Failed to connect");
        assert_eq!(entry.fields.get("service"), Some(&"api".to_string()));
    }

    #[test]
    fn test_level_filtering() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Error 1"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","message":"Info 1"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:32:00Z","level":"DEBUG","message":"Debug 1"}}"#).unwrap();

        let processor = LogProcessor::new(LogLevel::Warning);
        let entries = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, LogLevel::Error);
    }

    #[test]
    fn test_field_filtering() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Error 1","service":"api"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Error 2","component":"db"}}"#).unwrap();

        let processor = LogProcessor::new(LogLevel::Error)
            .with_include_fields(vec!["service".to_string()]);
        
        let entries = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].fields.get("service"), Some(&"api".to_string()));
    }
}