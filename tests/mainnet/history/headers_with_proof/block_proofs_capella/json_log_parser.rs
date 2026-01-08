use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: String,
    include_fields: Vec<String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            include_fields: Vec::new(),
        }
    }

    pub fn with_fields(mut self, fields: &[&str]) -> Self {
        self.include_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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
        let json: Value = serde_json::from_str(line)?;
        
        let timestamp = json["timestamp"]
            .as_str()
            .ok_or("Missing timestamp")?
            .parse::<DateTime<Utc>>()?;
        
        let level = json["level"]
            .as_str()
            .ok_or("Missing level")?
            .to_string();
        
        let message = json["message"]
            .as_str()
            .ok_or("Missing message")?
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if !["timestamp", "level", "message"].contains(&key.as_str()) {
                    if self.include_fields.is_empty() || self.include_fields.contains(key) {
                        fields.insert(key.clone(), value.clone());
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
        let level_order = |lvl: &str| match lvl.to_lowercase().as_str() {
            "error" => 4,
            "warn" => 3,
            "info" => 2,
            "debug" => 1,
            "trace" => 0,
            _ => 0,
        };

        level_order(&entry.level) >= level_order(&self.min_level)
    }

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!(
            "[{}] {}: {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.level.to_uppercase(),
            entry.message
        );

        if !entry.fields.is_empty() {
            let fields_str: Vec<String> = entry.fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            output.push_str(&format!(" ({})", fields_str.join(", ")));
        }

        output
    }
}

pub fn analyze_logs(entries: &[LogEntry]) -> HashMap<String, usize> {
    let mut stats = HashMap::new();
    
    for entry in entries {
        *stats.entry(entry.level.clone()).or_insert(0) += 1;
    }
    
    stats
}