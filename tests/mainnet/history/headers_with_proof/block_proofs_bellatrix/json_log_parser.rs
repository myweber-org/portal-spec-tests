use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
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

        for line in reader.lines() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }

            let json_value: Value = serde_json::from_str(&line_content)?;
            logs.push(json_value);
        }

        Ok(logs)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<Value>, LogParseError> {
        let logs = self.parse_logs()?;
        let filtered: Vec<Value> = logs
            .into_iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|l| l.eq_ignore_ascii_case(level))
                    .unwrap_or(false)
            })
            .collect();

        Ok(filtered)
    }

    pub fn extract_timestamps(&self) -> Result<Vec<String>, LogParseError> {
        let logs = self.parse_logs()?;
        let mut timestamps = Vec::new();

        for log in logs {
            if let Some(timestamp) = log.get("timestamp").and_then(|v| v.as_str()) {
                timestamps.push(timestamp.to_string());
            }
        }

        Ok(timestamps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "timestamp": "2023-10-01T12:00:00Z", "message": "System started"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "timestamp": "2023-10-01T12:05:00Z", "message": "Connection failed"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let logs = parser.parse_logs().unwrap();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "timestamp": "2023-10-01T12:00:00Z", "message": "Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "timestamp": "2023-10-01T12:05:00Z", "message": "Error"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["level"], "ERROR");
    }
}