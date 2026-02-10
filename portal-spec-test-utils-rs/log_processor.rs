use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    message: String,
    source: String,
}

impl LogEntry {
    fn new(timestamp: DateTime<FixedOffset>, level: &str, message: &str, source: &str) -> Self {
        LogEntry {
            timestamp,
            level: level.to_string(),
            message: message.to_string(),
            source: source.to_string(),
        }
    }
}

struct LogProcessor {
    entries: Vec<LogEntry>,
    filtered_entries: Vec<LogEntry>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            filtered_entries: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        self.filtered_entries = self.entries.clone();
        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        match DateTime::parse_from_rfc3339(parts[0].trim()) {
            Ok(timestamp) => Some(LogEntry::new(
                timestamp,
                parts[1].trim(),
                parts[2].trim(),
                parts[3].trim(),
            )),
            Err(_) => None,
        }
    }

    fn filter_by_level(&mut self, level: &str) {
        self.filtered_entries = self
            .entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .cloned()
            .collect();
    }

    fn filter_by_source(&mut self, source: &str) {
        self.filtered_entries = self
            .entries
            .iter()
            .filter(|entry| entry.source.contains(source))
            .cloned()
            .collect();
    }

    fn filter_by_time_range(
        &mut self,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    ) {
        self.filtered_entries = self
            .entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .cloned()
            .collect();
    }

    fn export_filtered<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;
        for entry in &self.filtered_entries {
            writeln!(
                file,
                "{} | {} | {} | {}",
                entry.timestamp.to_rfc3339(),
                entry.level,
                entry.message,
                entry.source
            )?;
        }
        Ok(())
    }

    fn get_statistics(&self) -> (usize, usize, Vec<(String, usize)>) {
        let total = self.entries.len();
        let filtered = self.filtered_entries.len();
        
        let mut level_counts = std::collections::HashMap::new();
        for entry in &self.filtered_entries {
            *level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        let mut level_vec: Vec<(String, usize)> = level_counts.into_iter().collect();
        level_vec.sort_by(|a, b| b.1.cmp(&a.1));
        
        (total, filtered, level_vec)
    }
}

fn main() -> io::Result<()> {
    let mut processor = LogProcessor::new();
    
    match processor.load_from_file("application.log") {
        Ok(_) => println!("Log file loaded successfully"),
        Err(e) => {
            eprintln!("Failed to load log file: {}", e);
            return Ok(());
        }
    }
    
    processor.filter_by_level("ERROR");
    
    let (total, filtered, stats) = processor.get_statistics();
    println!("Total entries: {}", total);
    println!("Filtered entries: {}", filtered);
    println!("Level distribution:");
    for (level, count) in stats {
        println!("  {}: {}", level, count);
    }
    
    processor.export_filtered("filtered_errors.log")?;
    println!("Filtered logs exported to filtered_errors.log");
    
    Ok(())
}
use chrono::{DateTime, Utc};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub component: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    timestamp_pattern: Regex,
    level_pattern: Regex,
    component_pattern: Regex,
}

impl LogProcessor {
    pub fn new() -> Result<Self, regex::Error> {
        Ok(LogProcessor {
            timestamp_pattern: Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")?,
            level_pattern: Regex::new(r"\[(INFO|WARN|ERROR|DEBUG)\]")?,
            component_pattern: Regex::new(r"\[([a-zA-Z0-9_\-\.]+)\]")?,
        })
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let timestamp = self.extract_timestamp(line)?;
        let level = self.extract_level(line).unwrap_or_else(|| "UNKNOWN".to_string());
        let component = self.extract_component(line).unwrap_or_else(|| "default".to_string());
        
        let message = self.clean_message(line);
        let metadata = self.extract_metadata(line);

        Some(LogEntry {
            timestamp,
            level,
            component,
            message,
            metadata,
        })
    }

    fn extract_timestamp(&self, line: &str) -> Option<DateTime<Utc>> {
        self.timestamp_pattern
            .find(line)
            .and_then(|m| DateTime::parse_from_rfc3339(m.as_str()).ok())
            .map(|dt| dt.with_timezone(&Utc))
    }

    fn extract_level(&self, line: &str) -> Option<String> {
        self.level_pattern
            .captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_component(&self, line: &str) -> Option<String> {
        self.component_pattern
            .captures_iter(line)
            .nth(1)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn clean_message(&self, line: &str) -> String {
        let cleaned = self.timestamp_pattern.replace_all(line, "").to_string();
        let cleaned = self.level_pattern.replace_all(&cleaned, "").to_string();
        let cleaned = self.component_pattern.replace_all(&cleaned, "").to_string();
        cleaned.trim().to_string()
    }

    fn extract_metadata(&self, line: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        let kv_pattern = Regex::new(r"([a-zA-Z_]+)=([a-zA-Z0-9_\-\.]+)").unwrap();
        
        for caps in kv_pattern.captures_iter(line) {
            if let (Some(key), Some(value)) = (caps.get(1), caps.get(2)) {
                metadata.insert(key.as_str().to_string(), value.as_str().to_string());
            }
        }
        metadata
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        for line in reader.lines() {
            if let Ok(line_content) = line {
                if let Some(entry) = self.parse_line(&line_content) {
                    entries.push(entry);
                }
            }
        }
        
        Ok(entries)
    }

    pub fn filter_by_level(&self, entries: &[LogEntry], level: &str) -> Vec<LogEntry> {
        entries
            .iter()
            .filter(|entry| entry.level == level)
            .cloned()
            .collect()
    }

    pub fn group_by_component(&self, entries: &[LogEntry]) -> HashMap<String, Vec<LogEntry>> {
        let mut groups = HashMap::new();
        for entry in entries {
            groups
                .entry(entry.component.clone())
                .or_insert_with(Vec::new)
                .push(entry.clone());
        }
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_log_line() {
        let processor = LogProcessor::new().unwrap();
        let line = "2024-01-15T10:30:45Z [INFO] [api_server] User login successful user_id=12345 session_id=abc123";
        
        let entry = processor.parse_line(line).unwrap();
        
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.component, "api_server");
        assert!(entry.message.contains("User login successful"));
        assert_eq!(entry.metadata.get("user_id"), Some(&"12345".to_string()));
        assert_eq!(entry.metadata.get("session_id"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_filter_by_level() {
        let processor = LogProcessor::new().unwrap();
        let entries = vec![
            LogEntry {
                timestamp: Utc::now(),
                level: "INFO".to_string(),
                component: "test".to_string(),
                message: "Test info".to_string(),
                metadata: HashMap::new(),
            },
            LogEntry {
                timestamp: Utc::now(),
                level: "ERROR".to_string(),
                component: "test".to_string(),
                message: "Test error".to_string(),
                metadata: HashMap::new(),
            },
        ];
        
        let filtered = processor.filter_by_level(&entries, "ERROR");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, "ERROR");
    }
}