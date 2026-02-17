use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidStructure(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<serde_json::Error> for LogError {
    fn from(err: serde_json::Error) -> Self {
        LogError::ParseError(err)
    }
}

pub struct LogProcessor {
    pub total_lines: usize,
    pub parsed_lines: usize,
    pub error_lines: usize,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            total_lines: 0,
            parsed_lines: 0,
            error_lines: 0,
        }
    }

    pub fn process_file(&mut self, path: &str) -> Result<Vec<Value>, LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            self.total_lines += 1;
            let line_content = line?;

            match self.parse_log_line(&line_content) {
                Ok(value) => {
                    self.parsed_lines += 1;
                    results.push(value);
                }
                Err(e) => {
                    self.error_lines += 1;
                    eprintln!("Failed to parse line {}: {:?}", self.total_lines, e);
                }
            }
        }

        Ok(results)
    }

    fn parse_log_line(&self, line: &str) -> Result<Value, LogError> {
        let value: Value = serde_json::from_str(line)?;

        if !value.is_object() {
            return Err(LogError::InvalidStructure(
                "Log line must be a JSON object".to_string(),
            ));
        }

        Ok(value)
    }

    pub fn statistics(&self) -> String {
        format!(
            "Processed {} lines: {} parsed successfully, {} errors",
            self.total_lines, self.parsed_lines, self.error_lines
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_json_logs() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-01", "level": "INFO", "message": "Test"}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-02", "level": "ERROR", "message": "Failed"}}"#).unwrap();

        let mut processor = LogProcessor::new();
        let results = processor.process_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(processor.total_lines, 2);
        assert_eq!(processor.parsed_lines, 2);
        assert_eq!(processor.error_lines, 0);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "not valid json").unwrap();

        let mut processor = LogProcessor::new();
        let results = processor.process_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(processor.total_lines, 1);
        assert_eq!(processor.parsed_lines, 0);
        assert_eq!(processor.error_lines, 1);
        assert_eq!(results.len(), 0);
    }
}