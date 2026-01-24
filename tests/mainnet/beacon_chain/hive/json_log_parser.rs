use serde_json::Value;
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
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp = json_value["timestamp"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let level = json_value["level"]
            .as_str()
            .unwrap_or("INFO")
            .to_string();

        let message = json_value["message"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
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

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }

    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = LogParser::new();
        assert_eq!(parser.get_entries().len(), 0);
    }

    #[test]
    fn test_line_parsing() {
        let parser = LogParser::new();
        let json_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","error_code":500}"#;
        
        let entry = parser.parse_line(json_line).unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.fields.get("error_code").unwrap().as_u64().unwrap(), 500);
    }
}