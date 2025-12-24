use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warning" => Some(LogLevel::Warning),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            "trace" => Some(LogLevel::Trace),
            _ => None,
        }
    }

    pub fn severity(&self) -> u8 {
        match self {
            LogLevel::Error => 5,
            LogLevel::Warning => 4,
            LogLevel::Info => 3,
            LogLevel::Debug => 2,
            LogLevel::Trace => 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn with_fields(mut self, fields: Vec<&str>) -> Self {
        self.include_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn exclude_patterns(mut self, patterns: Vec<&str>) -> Self {
        self.exclude_patterns = patterns.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                if self.should_include(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let parsed: serde_json::Value = serde_json::from_str(line)?;
        
        let timestamp = parsed["timestamp"]
            .as_str()
            .ok_or("Missing timestamp")?
            .to_string();

        let level_str = parsed["level"]
            .as_str()
            .ok_or("Missing log level")?;
        let level = LogLevel::from_str(level_str)
            .ok_or_else(|| format!("Invalid log level: {}", level_str))?;

        let message = parsed["message"]
            .as_str()
            .ok_or("Missing message")?
            .to_string();

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

        Ok(LogEntry {
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

        if !self.include_fields.is_empty() {
            for field in &self.include_fields {
                if !entry.fields.contains_key(field) {
                    return false;
                }
            }
        }

        for pattern in &self.exclude_patterns {
            if entry.message.contains(pattern) {
                return false;
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

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("unknown"), None);
    }

    #[test]
    fn test_log_level_severity() {
        assert!(LogLevel::Error.severity() > LogLevel::Warning.severity());
        assert!(LogLevel::Info.severity() > LogLevel::Debug.severity());
    }
}