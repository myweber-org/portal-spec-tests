
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct LogProcessor {
    pub file_path: String,
}

impl LogProcessor {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_logs(&self) -> Result<Vec<Value>, LogError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut parsed_logs = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            match self.parse_log_line(&line_content) {
                Ok(log_entry) => parsed_logs.push(log_entry),
                Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
            }
        }

        Ok(parsed_logs)
    }

    fn parse_log_line(&self, line: &str) -> Result<Value, LogError> {
        let parsed: Value = serde_json::from_str(line)?;
        
        if let Some(timestamp) = parsed.get("timestamp") {
            if timestamp.is_null() {
                return Err(LogError::MissingField("timestamp".to_string()));
            }
        } else {
            return Err(LogError::MissingField("timestamp".to_string()));
        }

        Ok(parsed)
    }

    pub fn filter_by_level(logs: &[Value], level: &str) -> Vec<Value> {
        logs.iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|l| l.as_str())
                    .map(|l| l == level)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let temp_file = NamedTempFile::new().unwrap();
        let log_line = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "Test log"}"#;
        std::fs::write(temp_file.path(), log_line).unwrap();

        let processor = LogProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_logs().unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["level"], "INFO");
    }

    #[test]
    fn test_filter_logs_by_level() {
        let logs = vec![
            json!({"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "Test 1"}),
            json!({"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "message": "Test 2"}),
            json!({"timestamp": "2024-01-15T10:32:00Z", "level": "INFO", "message": "Test 3"}),
        ];

        let filtered = LogProcessor::filter_by_level(&logs, "INFO");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|log| log["level"] == "INFO"));
    }
}