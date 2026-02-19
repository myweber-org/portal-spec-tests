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
}use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn parse_log_file(file_path: &str, min_level: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut filtered_logs = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(log_entry) = serde_json::from_str::<Value>(&line) {
            if let Some(level) = log_entry.get("level").and_then(|v| v.as_str()) {
                if should_include_log(level, min_level) {
                    filtered_logs.push(log_entry);
                }
            }
        }
    }

    Ok(filtered_logs)
}

fn should_include_log(log_level: &str, min_level: &str) -> bool {
    let levels = ["trace", "debug", "info", "warn", "error"];
    let log_idx = levels.iter().position(|&l| l == log_level);
    let min_idx = levels.iter().position(|&l| l == min_level);

    match (log_idx, min_idx) {
        (Some(l), Some(m)) => l >= m,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_filtering() {
        assert!(should_include_log("error", "info"));
        assert!(!should_include_log("debug", "warn"));
        assert!(should_include_log("warn", "warn"));
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct LogParser {
    pub entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut parser = LogParser::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str(&line) {
                parser.entries.push(entry);
            }
        }

        Ok(parser)
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .cloned()
            .collect()
    }

    pub fn filter_by_service(&self, service: &str) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .cloned()
            .collect()
    }

    pub fn search_in_message(&self, keyword: &str) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .cloned()
            .collect()
    }

    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        for entry in &self.entries {
            *stats.entry(entry.level.clone()).or_insert(0) += 1;
            *stats.entry(entry.service.clone()).or_insert(0) += 1;
        }
        
        stats
    }

    pub fn format_as_table(&self) -> String {
        let mut output = String::new();
        output.push_str("Timestamp | Level | Service | Message\n");
        output.push_str("----------|-------|---------|--------\n");
        
        for entry in &self.entries {
            let truncated_message = if entry.message.len() > 50 {
                format!("{}...", &entry.message[..47])
            } else {
                entry.message.clone()
            };
            
            output.push_str(&format!(
                "{} | {} | {} | {}\n",
                entry.timestamp, entry.level, entry.service, truncated_message
            ));
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parser() {
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"auth","message":"Authentication failed","user_id":123}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"api","message":"Request processed","duration_ms":45}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"database","message":"Slow query detected","query_time":2.5}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();
        
        let parser = LogParser::from_file(temp_file.path()).unwrap();
        assert_eq!(parser.entries.len(), 3);
        
        let errors = parser.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].service, "auth");
        
        let stats = parser.get_stats();
        assert_eq!(stats.get("ERROR"), Some(&1));
        assert_eq!(stats.get("auth"), Some(&1));
    }
}