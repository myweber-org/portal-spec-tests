use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidLogFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: Value,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let parsed: Value = serde_json::from_str(&line_content)?;
            
            let entry = LogEntry {
                timestamp: parsed["timestamp"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                level: parsed["level"]
                    .as_str()
                    .unwrap_or("INFO")
                    .to_string(),
                message: parsed["message"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                fields: parsed["fields"].clone(),
            };

            entries.push(entry);
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_uppercase() == level.to_uppercase())
            .collect();
        
        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "message": "Database connection failed", "fields": {"attempt": 3}}
{"timestamp": "2024-01-15T10:31:00Z", "level": "INFO", "message": "Service started", "fields": {"port": 8080}}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "ERROR");
        assert_eq!(entries[1].level, "INFO");
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "message": "Error 1", "fields": {}}
{"timestamp": "2024-01-15T10:31:00Z", "level": "INFO", "message": "Info 1", "fields": {}}
{"timestamp": "2024-01-15T10:32:00Z", "level": "ERROR", "message": "Error 2", "fields": {}}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.level == "ERROR"));
    }
}