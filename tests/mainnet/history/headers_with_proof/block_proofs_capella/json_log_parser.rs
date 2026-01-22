use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct JsonLogParser {
    file_path: String,
}

impl JsonLogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse_logs(&self) -> Result<Vec<Value>, LogParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            let json_value: Value = serde_json::from_str(&line_content)?;
            
            if !json_value.is_object() {
                return Err(LogParseError::MissingField(
                    format!("Line {}: Expected JSON object", line_num + 1)
                ));
            }
            
            logs.push(json_value);
        }

        Ok(logs)
    }

    pub fn filter_by_level(&self, logs: &[Value], level: &str) -> Vec<Value> {
        logs.iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|l| l.eq_ignore_ascii_case(level))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn extract_timestamps(&self, logs: &[Value]) -> Vec<String> {
        logs.iter()
            .filter_map(|log| {
                log.get("timestamp")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "System started"}
{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "message": "Connection failed"}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::write(&mut temp_file, log_data).unwrap();
        
        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let logs = parser.parse_logs().unwrap();
        
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0]["level"], "INFO");
        assert_eq!(logs[1]["level"], "ERROR");
    }

    #[test]
    fn test_filter_by_level() {
        let logs = vec![
            json!({"level": "INFO", "message": "test1"}),
            json!({"level": "ERROR", "message": "test2"}),
            json!({"level": "INFO", "message": "test3"}),
        ];
        
        let parser = JsonLogParser::new("dummy.log");
        let filtered = parser.filter_by_level(&logs, "INFO");
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0]["message"], "test1");
        assert_eq!(filtered[1]["message"], "test3");
    }
}