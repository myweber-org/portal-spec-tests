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
    filter_level: Option<String>,
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filter_level: None,
            required_fields: Vec::new(),
        }
    }

    pub fn set_level_filter(&mut self, level: &str) -> &mut Self {
        self.filter_level = Some(level.to_lowercase());
        self
    }

    pub fn add_required_field(&mut self, field: &str) -> &mut Self {
        self.required_fields.push(field.to_string());
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, String> {
        let json_value: Value = serde_json::from_str(line)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        let obj = json_value.as_object()
            .ok_or_else(|| "Log line is not a JSON object".to_string())?;

        let mut entry = LogEntry {
            timestamp: obj.get("timestamp").and_then(|v| v.as_str()).map(|s| s.to_string()),
            level: obj.get("level").and_then(|v| v.as_str()).map(|s| s.to_lowercase()),
            message: obj.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()),
            fields: HashMap::new(),
        };

        for (key, value) in obj {
            match key.as_str() {
                "timestamp" | "level" | "message" => continue,
                _ => {
                    entry.fields.insert(key.clone(), value.clone());
                }
            }
        }

        if let Some(filter) = &self.filter_level {
            if let Some(entry_level) = &entry.level {
                if entry_level != filter {
                    return Err("Level filter mismatch".to_string());
                }
            }
        }

        for field in &self.required_fields {
            if !obj.contains_key(field) {
                return Err(format!("Missing required field: {}", field));
            }
        }

        Ok(entry)
    }

    pub fn extract_field_values(&self, entries: &[LogEntry], field_name: &str) -> Vec<Value> {
        entries.iter()
            .filter_map(|entry| entry.fields.get(field_name).cloned())
            .collect()
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let mut parser = LogParser::new();
        parser.set_level_filter("error");

        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"error","message":"Database connection failed","error_code":500,"service":"api"}"#;
        
        let entry = parser.parse_line(log_data).unwrap();
        assert_eq!(entry.level.unwrap(), "error");
        assert_eq!(entry.message.unwrap(), "Database connection failed");
        assert_eq!(entry.fields.get("error_code").unwrap().as_u64().unwrap(), 500);
    }

    #[test]
    fn test_missing_required_field() {
        let mut parser = LogParser::new();
        parser.add_required_field("user_id");

        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"User logged in"}"#;
        
        let result = parser.parse_line(log_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field"));
    }

    #[test]
    fn test_parse_log_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let logs = vec![
            r#"{"timestamp":"2024-01-15T10:30:00Z","level":"error","message":"DB error","code":500}"#,
            r#"{"timestamp":"2024-01-15T10:31:00Z","level":"info","message":"Request processed","duration":150}"#,
            r#"{"timestamp":"2024-01-15T10:32:00Z","level":"error","message":"Timeout","code":408}"#,
        ];

        for log in logs {
            writeln!(temp_file, "{}", log).unwrap();
        }

        let parser = LogParser::new();
        let entries = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 3);
        
        let error_entries: Vec<_> = entries.iter()
            .filter(|e| e.level.as_deref() == Some("error"))
            .collect();
        assert_eq!(error_entries.len(), 2);
    }
}use serde_json::Value;
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
    min_level: Option<String>,
    filter_fields: HashMap<String, Value>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            filter_fields: HashMap::new(),
        }
    }

    pub fn set_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    pub fn add_filter(mut self, key: &str, value: Value) -> Self {
        self.filter_fields.insert(key.to_string(), value);
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
            .unwrap_or("")
            .to_string();

        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_lowercase();

        if let Some(min_level) = &self.min_level {
            if !self.is_level_allowed(&level, min_level) {
                return Err("Log level below minimum threshold".into());
            }
        }

        let message = json_value.get("message")
            .and_then(|v| v.as_str())
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

        for (filter_key, filter_value) in &self.filter_fields {
            if let Some(entry_value) = fields.get(filter_key) {
                if entry_value != filter_value {
                    return Err("Field filter mismatch".into());
                }
            } else {
                return Err("Required field not found".into());
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn is_level_allowed(&self, entry_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error", "fatal"];
        let entry_idx = levels.iter().position(|&l| l == entry_level);
        let min_idx = levels.iter().position(|&l| l == min_level);

        match (entry_idx, min_idx) {
            (Some(e), Some(m)) => e >= m,
            _ => true,
        }
    }
}

impl LogEntry {
    pub fn format(&self, include_fields: bool) -> String {
        let mut output = format!("[{}] {}: {}", self.timestamp, self.level.to_uppercase(), self.message);
        
        if include_fields && !self.fields.is_empty() {
            output.push_str(" | ");
            let fields_str: Vec<String> = self.fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            output.push_str(&fields_str.join(", "));
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_log() {
        let parser = LogParser::new();
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"Service started","service":"api","version":"1.0.0"}"#;
        
        let entry = parser.parse_line(log_line).unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "Service started");
        assert_eq!(entry.fields.get("service").unwrap().as_str().unwrap(), "api");
    }

    #[test]
    fn test_level_filtering() {
        let parser = LogParser::new().set_min_level("warn");
        
        let warn_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"warn","message":"High memory usage"}"#;
        assert!(parser.parse_line(warn_log).is_ok());
        
        let info_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"Service started"}"#;
        assert!(parser.parse_line(info_log).is_err());
    }
}