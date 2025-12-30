
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
    pub entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        
        for line_result in reader.lines() {
            let line = line_result?;
            
            if line.trim().is_empty() {
                continue;
            }
            
            let entry = self.parse_line(&line)?;
            self.entries.push(entry);
            count += 1;
        }
        
        Ok(count)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, ParseError> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp = json_value["timestamp"]
            .as_str()
            .ok_or_else(|| ParseError::InvalidLogFormat("Missing timestamp".to_string()))?
            .to_string();
            
        let level = json_value["level"]
            .as_str()
            .ok_or_else(|| ParseError::InvalidLogFormat("Missing level".to_string()))?
            .to_string();
            
        let message = json_value["message"]
            .as_str()
            .ok_or_else(|| ParseError::InvalidLogFormat("Missing message".to_string()))?
            .to_string();
        
        let mut fields = json_value.clone();
        if let Some(obj) = fields.as_object_mut() {
            obj.remove("timestamp");
            obj.remove("level");
            obj.remove("message");
        }
        
        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn get_stats(&self) -> (usize, usize, usize) {
        let error_count = self.filter_by_level("error").len();
        let warn_count = self.filter_by_level("warn").len();
        let info_count = self.filter_by_level("info").len();
        
        (error_count, warn_count, info_count)
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
        
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Failed to connect","service":"api","attempt":3}"#;
        
        let entry = parser.parse_line(log_data).unwrap();
        
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Failed to connect");
        assert_eq!(entry.fields["service"], "api");
        assert_eq!(entry.fields["attempt"], 3);
    }

    #[test]
    fn test_parse_file() {
        let mut parser = LogParser::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Failed to connect"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","message":"Connection established"}}"#).unwrap();
        
        let count = parser.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(count, 2);
        assert_eq!(parser.entries.len(), 2);
        
        let stats = parser.get_stats();
        assert_eq!(stats, (1, 0, 1));
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Failed".to_string(),
                fields: Value::Object(serde_json::Map::new()),
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Success".to_string(),
                fields: Value::Object(serde_json::Map::new()),
            },
            LogEntry {
                timestamp: "2024-01-15T10:32:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Failed again".to_string(),
                fields: Value::Object(serde_json::Map::new()),
            },
        ];
        
        parser.entries = entries;
        
        let errors = parser.filter_by_level("error");
        assert_eq!(errors.len(), 2);
        
        let infos = parser.filter_by_level("info");
        assert_eq!(infos.len(), 1);
    }
}