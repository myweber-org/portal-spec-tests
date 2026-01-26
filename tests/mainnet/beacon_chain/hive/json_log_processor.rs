
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogProcessor {
    min_level: String,
    filter_fields: Vec<String>,
}

impl LogProcessor {
    pub fn new(min_level: &str) -> Self {
        LogProcessor {
            min_level: min_level.to_lowercase(),
            filter_fields: Vec::new(),
        }
    }

    pub fn add_filter_field(&mut self, field: &str) {
        self.filter_fields.push(field.to_string());
    }

    pub fn process_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
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

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn Error>> {
        let json: Value = serde_json::from_str(line)?;

        let timestamp = json["timestamp"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let level = json["level"]
            .as_str()
            .unwrap_or("info")
            .to_string()
            .to_lowercase();

        let message = json["message"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if !matches!(key.as_str(), "timestamp" | "level" | "message") {
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

    fn should_include(&self, entry: &LogEntry) -> bool {
        let level_order = |level: &str| match level {
            "error" => 4,
            "warn" => 3,
            "info" => 2,
            "debug" => 1,
            _ => 0,
        };

        let entry_level = level_order(&entry.level);
        let min_level = level_order(&self.min_level);

        if entry_level < min_level {
            return false;
        }

        if self.filter_fields.is_empty() {
            return true;
        }

        self.filter_fields.iter().any(|field| {
            entry.fields.contains_key(field)
        })
    }
}

pub fn count_errors(entries: &[LogEntry]) -> usize {
    entries.iter()
        .filter(|entry| entry.level == "error")
        .count()
}

pub fn extract_field_values(entries: &[LogEntry], field: &str) -> Vec<Value> {
    entries.iter()
        .filter_map(|entry| entry.fields.get(field))
        .cloned()
        .collect()
}