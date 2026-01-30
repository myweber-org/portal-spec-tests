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
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let json_value: Value = serde_json::from_str(line).ok()?;
        
        let mut fields = HashMap::new();
        let mut timestamp = None;
        let mut level = None;
        let mut message = None;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match key.as_str() {
                    "timestamp" | "time" | "@timestamp" => {
                        timestamp = value.as_str().map(|s| s.to_string());
                    }
                    "level" | "log_level" | "severity" => {
                        level = value.as_str().map(|s| s.to_lowercase());
                    }
                    "message" | "msg" | "log" => {
                        message = value.as_str().map(|s| s.to_string());
                    }
                    _ => {
                        fields.insert(key, value);
                    }
                }
            }
        }

        if let Some(filter) = &self.filter_level {
            if level.as_deref() != Some(filter) {
                return None;
            }
        }

        if !self.required_fields.is_empty() {
            for field in &self.required_fields {
                if !fields.contains_key(field) {
                    return None;
                }
            }
        }

        Some(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn extract_field_values(&self, entries: &[LogEntry], field_name: &str) -> Vec<Value> {
        entries
            .iter()
            .filter_map(|entry| entry.fields.get(field_name).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_with_filter() {
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "message": "Failed to connect", "service": "api"}"#;
        
        let mut parser = LogParser::new();
        parser.set_level_filter("error");
        
        let entry = parser.parse_line(log_data).unwrap();
        assert_eq!(entry.level, Some("error".to_string()));
        assert_eq!(entry.message, Some("Failed to connect".to_string()));
        assert_eq!(entry.fields.get("service").and_then(|v| v.as_str()), Some("api"));
    }

    #[test]
    fn test_field_extraction() {
        let entries = vec![
            LogEntry {
                timestamp: Some("2024-01-15T10:30:00Z".to_string()),
                level: Some("info".to_string()),
                message: Some("Request processed".to_string()),
                fields: vec![("user_id".to_string(), Value::String("user123".to_string()))]
                    .into_iter()
                    .collect(),
            },
        ];
        
        let parser = LogParser::new();
        let user_ids = parser.extract_field_values(&entries, "user_id");
        
        assert_eq!(user_ids.len(), 1);
        assert_eq!(user_ids[0].as_str(), Some("user123"));
    }
}