
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct LogFilter {
    pub min_level: Option<String>,
    pub contains_text: Option<String>,
    pub field_filters: HashMap<String, Value>,
}

impl LogFilter {
    pub fn new() -> Self {
        LogFilter {
            min_level: None,
            contains_text: None,
            field_filters: HashMap::new(),
        }
    }

    pub fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_string());
        self
    }

    pub fn with_text_filter(mut self, text: &str) -> Self {
        self.contains_text = Some(text.to_string());
        self
    }

    pub fn with_field_filter(mut self, key: &str, value: Value) -> Self {
        self.field_filters.insert(key.to_string(), value);
        self
    }

    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            if !self.compare_levels(&entry.level, min_level) {
                return false;
            }
        }

        if let Some(text) = &self.contains_text {
            if !entry.message.contains(text) {
                return false;
            }
        }

        for (key, filter_value) in &self.field_filters {
            if let Some(entry_value) = entry.fields.get(key) {
                if entry_value != filter_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn compare_levels(&self, entry_level: &str, min_level: &str) -> bool {
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
            if let Ok(entry) = self.parse_line(&line) {
                if self.filter.matches(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_string();

        let message = json_value.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if !["timestamp", "level", "message"].contains(&key.as_str()) {
                    fields.insert(key.clone(), value.clone());
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

    pub fn extract_fields(&self, entries: &[LogEntry], field_names: &[&str]) -> Vec<HashMap<String, Value>> {
        entries.iter()
            .map(|entry| {
                let mut extracted = HashMap::new();
                for field_name in field_names {
                    if let Some(value) = entry.fields.get(*field_name) {
                        extracted.insert(field_name.to_string(), value.clone());
                    }
                }
                extracted
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_log_filter() {
        let filter = LogFilter::new()
            .with_min_level("info")
            .with_text_filter("error")
            .with_field_filter("service", json!("api"));

        let mut fields = HashMap::new();
        fields.insert("service".to_string(), json!("api"));
        
        let entry = LogEntry {
            timestamp: "2023-10-01T12:00:00Z".to_string(),
            level: "error".to_string(),
            message: "Database connection error".to_string(),
            fields,
        };

        assert!(filter.matches(&entry));
    }

    #[test]
    fn test_level_comparison() {
        let filter = LogFilter::new().with_min_level("warn");
        
        let entry = LogEntry {
            timestamp: "2023-10-01T12:00:00Z".to_string(),
            level: "error".to_string(),
            message: "Test".to_string(),
            fields: HashMap::new(),
        };

        assert!(filter.matches(&entry));
    }
}