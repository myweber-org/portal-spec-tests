
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: Option<NaiveDateTime>,
    pub level: String,
    pub message: String,
}

pub struct LogParser {
    timestamp_pattern: Regex,
    level_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            timestamp_pattern: Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap(),
            level_pattern: Regex::new(r"(ERROR|WARN|INFO|DEBUG)").unwrap(),
        }
    }

    pub fn parse_file(&self, path: &str) -> io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let timestamp = self.extract_timestamp(line);
        let level = self.extract_level(line).unwrap_or_else(|| "UNKNOWN".to_string());
        
        if level == "ERROR" || level == "WARN" {
            Some(LogEntry {
                timestamp,
                level,
                message: line.to_string(),
            })
        } else {
            None
        }
    }

    fn extract_timestamp(&self, line: &str) -> Option<NaiveDateTime> {
        self.timestamp_pattern.find(line)
            .and_then(|m| NaiveDateTime::parse_from_str(m.as_str(), "%Y-%m-%d %H:%M:%S").ok())
    }

    fn extract_level(&self, line: &str) -> Option<String> {
        self.level_pattern.find(line)
            .map(|m| m.as_str().to_string())
    }
}

pub fn filter_errors(entries: &[LogEntry]) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level == "ERROR")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_line() {
        let parser = LogParser::new();
        let line = "2023-10-05 14:30:25 ERROR Database connection failed";
        
        let entry = parser.parse_line(line).unwrap();
        assert_eq!(entry.level, "ERROR");
        assert!(entry.timestamp.is_some());
        assert!(entry.message.contains("Database connection failed"));
    }

    #[test]
    fn test_filter_errors() {
        let entries = vec![
            LogEntry {
                timestamp: None,
                level: "INFO".to_string(),
                message: "System started".to_string(),
            },
            LogEntry {
                timestamp: None,
                level: "ERROR".to_string(),
                message: "Critical failure".to_string(),
            },
        ];

        let errors = filter_errors(&entries);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }
}