use serde::{Deserialize, Serialize};
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
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if line.trim().is_empty() {
                continue;
            }
            
            let entry: LogEntry = serde_json::from_str(&line)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            
            self.entries.push(entry);
            count += 1;
        }
        
        Ok(count)
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

    pub fn summarize(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
            *summary.entry(entry.service.clone()).or_insert(0) += 1;
        }
        
        summary
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .cloned()
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "service": "auth", "message": "Authentication failed", "metadata": {{"user": "john", "ip": "192.168.1.1"}}}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:31:00Z", "level": "INFO", "service": "api", "message": "Request processed", "metadata": {{"endpoint": "/users", "duration": "150ms"}}}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:32:00Z", "level": "ERROR", "service": "auth", "message": "Invalid token", "metadata": {{"user": "jane", "ip": "192.168.1.2"}}}}"#).unwrap();
        file
    }

    #[test]
    fn test_load_logs() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        
        let count = parser.load_from_file(file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(parser.get_entries().len(), 3);
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        let errors = parser.filter_by_level("ERROR");
        assert_eq!(errors.len(), 2);
        
        let infos = parser.filter_by_level("INFO");
        assert_eq!(infos.len(), 1);
    }

    #[test]
    fn test_filter_by_service() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        let auth_logs = parser.filter_by_service("auth");
        assert_eq!(auth_logs.len(), 2);
        
        let api_logs = parser.filter_by_service("api");
        assert_eq!(api_logs.len(), 1);
    }

    #[test]
    fn test_summarize() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        let summary = parser.summarize();
        assert_eq!(summary.get("ERROR"), Some(&2));
        assert_eq!(summary.get("INFO"), Some(&1));
        assert_eq!(summary.get("auth"), Some(&2));
        assert_eq!(summary.get("api"), Some(&1));
    }

    #[test]
    fn test_search_messages() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        let failed_logs = parser.search_messages("failed");
        assert_eq!(failed_logs.len(), 1);
        
        let processed_logs = parser.search_messages("processed");
        assert_eq!(processed_logs.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        assert_eq!(parser.get_entries().len(), 3);
        parser.clear();
        assert_eq!(parser.get_entries().len(), 0);
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

    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                let entry = self.parse_json_value(json_value);
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_json_value(&self, value: Value) -> LogEntry {
        let mut fields = HashMap::new();
        let mut timestamp = None;
        let mut level = None;
        let mut message = None;

        if let Value::Object(map) = value {
            for (key, val) in map {
                match key.as_str() {
                    "timestamp" | "time" | "@timestamp" => {
                        if let Value::String(s) = val {
                            timestamp = Some(s);
                        } else {
                            fields.insert(key, val);
                        }
                    }
                    "level" | "log_level" | "severity" => {
                        if let Value::String(s) = val {
                            level = Some(s.to_uppercase());
                        } else {
                            fields.insert(key, val);
                        }
                    }
                    "message" | "msg" | "log" => {
                        if let Value::String(s) = val {
                            message = Some(s);
                        } else {
                            fields.insert(key, val);
                        }
                    }
                    _ => {
                        fields.insert(key, val);
                    }
                }
            }
        }

        LogEntry {
            timestamp,
            level,
            message,
            fields,
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        let target_level = level.to_uppercase();
        self.entries
            .iter()
            .filter(|entry| entry.level.as_ref().map_or(false, |l| l == &target_level))
            .collect()
    }

    pub fn extract_field(&self, field_name: &str) -> Vec<Option<&Value>> {
        self.entries
            .iter()
            .map(|entry| entry.fields.get(field_name))
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for entry in &self.entries {
            if let Some(level) = &entry.level {
                *counts.entry(level.clone()).or_insert(0) += 1;
            }
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_json_value() {
        let parser = LogParser::new();
        let json_data = json!({
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "error",
            "message": "Failed to connect to database",
            "service": "api",
            "error_code": 500
        });

        let entry = parser.parse_json_value(json_data);

        assert_eq!(entry.timestamp, Some("2024-01-15T10:30:00Z".to_string()));
        assert_eq!(entry.level, Some("ERROR".to_string()));
        assert_eq!(entry.message, Some("Failed to connect to database".to_string()));
        assert_eq!(entry.fields.get("service"), Some(&json!("api")));
        assert_eq!(entry.fields.get("error_code"), Some(&json!(500)));
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        
        let entry1 = LogEntry {
            timestamp: Some("2024-01-15T10:30:00Z".to_string()),
            level: Some("ERROR".to_string()),
            message: Some("Error 1".to_string()),
            fields: HashMap::new(),
        };
        
        let entry2 = LogEntry {
            timestamp: Some("2024-01-15T10:31:00Z".to_string()),
            level: Some("INFO".to_string()),
            message: Some("Info 1".to_string()),
            fields: HashMap::new(),
        };
        
        parser.entries.push(entry1);
        parser.entries.push(entry2);

        let errors = parser.filter_by_level("error");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, Some("ERROR".to_string()));
    }
}use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    message: String,
    module: Option<String>,
}

struct LogParser {
    file_path: String,
}

impl LogParser {
    fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    fn parse_logs(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }
            let log_entry: LogEntry = serde_json::from_str(&line_content)?;
            logs.push(log_entry);
        }

        Ok(logs)
    }

    fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let logs = self.parse_logs()?;
        let filtered: Vec<LogEntry> = logs
            .into_iter()
            .filter(|log| log.level == level)
            .collect();
        Ok(filtered)
    }

    fn count_logs_by_level(&self) -> Result<std::collections::HashMap<LogLevel, usize>, Box<dyn std::error::Error>> {
        let logs = self.parse_logs()?;
        let mut counts = std::collections::HashMap::new();

        for log in logs {
            *counts.entry(log.level).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = LogParser::new("logs.json");
    
    match parser.parse_logs() {
        Ok(logs) => println!("Total logs parsed: {}", logs.len()),
        Err(e) => eprintln!("Failed to parse logs: {}", e),
    }

    let error_logs = parser.filter_by_level(LogLevel::ERROR)?;
    println!("Error logs count: {}", error_logs.len());

    let level_counts = parser.count_logs_by_level()?;
    for (level, count) in level_counts {
        println!("{:?}: {}", level, count);
    }

    Ok(())
}