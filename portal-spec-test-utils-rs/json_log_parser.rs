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
}use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use chrono::{DateTime, FixedOffset};
use serde_json::Value;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
    pub component: Option<String>,
}

pub struct LogParser {
    min_level: String,
    start_time: Option<DateTime<FixedOffset>>,
    end_time: Option<DateTime<FixedOffset>>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            start_time: None,
            end_time: None,
        }
    }

    pub fn set_time_range(&mut self, start: Option<DateTime<FixedOffset>>, end: Option<DateTime<FixedOffset>>) {
        self.start_time = start;
        self.end_time = end;
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(line)?;

        let timestamp_str = json["timestamp"]
            .as_str()
            .ok_or("Missing timestamp field")?;
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;

        if let Some(start) = self.start_time {
            if timestamp < start {
                return Err("Before time range".into());
            }
        }

        if let Some(end) = self.end_time {
            if timestamp > end {
                return Err("After time range".into());
            }
        }

        let level = json["level"]
            .as_str()
            .ok_or("Missing level field")?
            .to_lowercase();

        let level_priority = self.get_level_priority(&level);
        let min_priority = self.get_level_priority(&self.min_level);

        if level_priority < min_priority {
            return Err("Below minimum log level".into());
        }

        let message = json["message"]
            .as_str()
            .ok_or("Missing message field")?
            .to_string();

        let component = json["component"].as_str().map(|s| s.to_string());

        Ok(LogEntry {
            timestamp,
            level,
            message,
            component,
        })
    }

    fn get_level_priority(&self, level: &str) -> u8 {
        match level {
            "trace" => 1,
            "debug" => 2,
            "info" => 3,
            "warn" => 4,
            "error" => 5,
            "fatal" => 6,
            _ => 0,
        }
    }
}

pub fn print_entries(entries: &[LogEntry]) {
    for entry in entries {
        println!(
            "[{}] {} - {} {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.level.to_uppercase(),
            entry.component.as_deref().unwrap_or("unknown"),
            entry.message
        );
    }
}