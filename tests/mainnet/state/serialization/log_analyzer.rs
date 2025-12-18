
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warning" => Some(LogLevel::Warning),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    source: String,
}

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<LogLevel, usize>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    fn parse_log_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        self.update_statistics();
        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let timestamp_str = parts[0].trim();
        let level_str = parts[1].trim();
        let source = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();

        let timestamp = match DateTime::parse_from_rfc3339(timestamp_str) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return None,
        };

        let level = LogLevel::from_str(level_str)?;

        Some(LogEntry {
            timestamp,
            level,
            message,
            source,
        })
    }

    fn update_statistics(&mut self) {
        self.level_counts.clear();
        for entry in &self.entries {
            *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
    }

    fn filter_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    fn get_error_count(&self) -> usize {
        *self.level_counts.get(&LogLevel::Error).unwrap_or(&0)
    }

    fn get_most_frequent_source(&self) -> Option<&String> {
        let mut source_counts: HashMap<&String, usize> = HashMap::new();
        
        for entry in &self.entries {
            *source_counts.entry(&entry.source).or_insert(0) += 1;
        }

        source_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(source, _)| *source)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = LogAnalyzer::new();
    
    analyzer.parse_log_file("application.log")?;
    
    println!("Total log entries: {}", analyzer.entries.len());
    println!("Error count: {}", analyzer.get_error_count());
    
    if let Some(source) = analyzer.get_most_frequent_source() {
        println!("Most frequent source: {}", source);
    }
    
    let errors = analyzer.filter_by_level(LogLevel::Error);
    println!("Recent errors:");
    for error in errors.iter().take(5) {
        println!("[{}] {}: {}", error.timestamp, error.source, error.message);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("unknown"), None);
    }

    #[test]
    fn test_log_line_parsing() {
        let analyzer = LogAnalyzer::new();
        let line = "2023-10-01T12:00:00Z | ERROR | auth_service | Failed to authenticate user";
        
        let entry = analyzer.parse_log_line(line).unwrap();
        
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.source, "auth_service");
        assert_eq!(entry.message, "Failed to authenticate user");
    }

    #[test]
    fn test_filter_by_level() {
        let mut analyzer = LogAnalyzer::new();
        
        analyzer.entries.push(LogEntry {
            timestamp: Utc.with_ymd_and_hms(2023, 10, 1, 12, 0, 0).unwrap(),
            level: LogLevel::Error,
            message: "Test error".to_string(),
            source: "test".to_string(),
        });
        
        analyzer.entries.push(LogEntry {
            timestamp: Utc.with_ymd_and_hms(2023, 10, 1, 12, 1, 0).unwrap(),
            level: LogLevel::Info,
            message: "Test info".to_string(),
            source: "test".to_string(),
        });
        
        let errors = analyzer.filter_by_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Test error");
    }
}