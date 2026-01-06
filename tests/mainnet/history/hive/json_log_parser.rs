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
    pub filter_level: Option<String>,
    pub extract_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filter_level: None,
            extract_fields: Vec::new(),
        }
    }

    pub fn with_level_filter(mut self, level: &str) -> Self {
        self.filter_level = Some(level.to_lowercase());
        self
    }

    pub fn with_field_extraction(mut self, fields: Vec<&str>) -> Self {
        self.extract_fields = fields.iter().map(|s| s.to_string()).collect();
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
        let mut entry = LogEntry {
            timestamp: json_value.get("timestamp").and_then(|v| v.as_str()).map(|s| s.to_string()),
            level: json_value.get("level").and_then(|v| v.as_str()).map(|s| s.to_lowercase()),
            message: json_value.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()),
            fields: HashMap::new(),
        };

        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if !["timestamp", "level", "message"].contains(&key.as_str()) {
                    if self.extract_fields.is_empty() || self.extract_fields.contains(key) {
                        entry.fields.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        if let Some(filter) = &self.filter_level {
            if let Some(level) = &entry.level {
                if level != filter {
                    return Err("Level filter mismatch".into());
                }
            }
        }

        Ok(entry)
    }
}

pub fn summarize_logs(entries: &[LogEntry]) -> HashMap<String, usize> {
    let mut summary = HashMap::new();
    
    for entry in entries {
        if let Some(level) = &entry.level {
            *summary.entry(level.clone()).or_insert(0) += 1;
        }
    }
    
    summary
}