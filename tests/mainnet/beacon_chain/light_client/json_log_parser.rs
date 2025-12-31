use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    filters: HashMap<String, String>,
    format_template: String,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: HashMap::new(),
            format_template: String::from("{timestamp} - {level} - {message}"),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn set_format(&mut self, format: &str) {
        self.format_template = format.to_string();
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.matches_filters(&json_value) {
                    if let Some(formatted) = self.format_entry(&json_value) {
                        results.push(formatted);
                    }
                }
            }
        }

        Ok(results)
    }

    fn matches_filters(&self, json: &Value) -> bool {
        for (key, expected_value) in &self.filters {
            if let Some(actual_value) = json.get(key) {
                if actual_value.as_str() != Some(expected_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn format_entry(&self, json: &Value) -> Option<String> {
        let mut result = self.format_template.clone();
        
        for (key, value) in json.as_object()? {
            let placeholder = format!("{{{}}}", key);
            if let Some(str_value) = value.as_str() {
                result = result.replace(&placeholder, str_value);
            } else {
                result = result.replace(&placeholder, &value.to_string());
            }
        }
        
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut parser = LogParser::new();
        parser.add_filter("level", "ERROR");
        parser.set_format("{timestamp} :: {level} :: {message}");

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2023-10-01T12:00:00Z", "level": "ERROR", "message": "Database connection failed"}}"#
        ).unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2023-10-01T12:01:00Z", "level": "INFO", "message": "User logged in"}}"#
        ).unwrap();

        let results = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "2023-10-01T12:00:00Z :: ERROR :: Database connection failed");
    }
}