use serde_json::Value;
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
            .ok_or("Log entry must be a JSON object")?;

        let timestamp = obj.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let level = obj.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_lowercase();

        if let Some(filter) = &self.filter_level {
            if &level != filter {
                return Err("Level filter mismatch".to_string());
            }
        }

        let message = obj.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        for (key, value) in obj {
            if key != "timestamp" && key != "level" && key != "message" {
                fields.insert(key.clone(), value.clone());
            }
        }

        for required_field in &self.required_fields {
            if !fields.contains_key(required_field) {
                return Err(format!("Missing required field: {}", required_field));
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn extract_field_values(&self, entries: &[LogEntry], field_name: &str) -> Vec<Value> {
        entries.iter()
            .filter_map(|entry| entry.fields.get(field_name))
            .cloned()
            .collect()
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
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let entries = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "error");
        assert_eq!(entries[0].message, "Database connection failed");
        assert_eq!(entries[0].fields.get("error_code").unwrap().as_i64(), Some(500));
    }

    #[test]
    fn test_level_filter() {
        let mut parser = LogParser::new();
        parser.set_level_filter("info");

        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"error","message":"Error message"}
{"timestamp":"2024-01-15T10:31:00Z","level":"info","message":"Info message"}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let entries = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "info");
    }
}