use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: HashMap<String, String>,
}

struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: LogEntry = serde_json::from_str(&line)?;
            self.entries.push(entry);
        }
        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    fn group_by_service(&self) -> HashMap<String, Vec<&LogEntry>> {
        let mut groups = HashMap::new();
        for entry in &self.entries {
            groups
                .entry(entry.service.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        groups
    }

    fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn search_in_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LogProcessor::new();
    processor.load_from_file("logs.jsonl")?;

    println!("Total entries: {}", processor.entries.len());

    let error_logs = processor.filter_by_level("ERROR");
    println!("Error logs: {}", error_logs.len());

    let service_groups = processor.group_by_service();
    for (service, logs) in service_groups {
        println!("Service '{}': {} logs", service, logs.len());
    }

    let level_counts = processor.count_by_level();
    for (level, count) in level_counts {
        println!("Level {}: {} occurrences", level, count);
    }

    let search_results = processor.search_in_messages("timeout");
    println!("Found {} logs containing 'timeout'", search_results.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"ERROR","service":"api","message":"Connection timeout","metadata":{"ip":"192.168.1.1"}}
{"timestamp":"2023-10-01T12:01:00Z","level":"INFO","service":"auth","message":"User login successful","metadata":{"user_id":"123"}}
{"timestamp":"2023-10-01T12:02:00Z","level":"WARN","service":"api","message":"High latency detected","metadata":{}}"#;
        temp_file.write_all(log_data.as_bytes()).unwrap();

        let mut processor = LogProcessor::new();
        processor.load_from_file(temp_file.path()).unwrap();

        assert_eq!(processor.entries.len(), 3);
        assert_eq!(processor.filter_by_level("ERROR").len(), 1);
        assert_eq!(processor.search_in_messages("timeout").len(), 1);
        
        let counts = processor.count_by_level();
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("INFO"), Some(&1));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    service_filter: Option<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            service_filter: None,
        }
    }

    pub fn with_service_filter(mut self, service_name: &str) -> Self {
        self.service_filter = Some(service_name.to_string());
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.should_process(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn should_process(&self, entry: &LogEntry) -> bool {
        if self.level_to_numeric(&entry.level) < self.level_to_numeric(&self.min_level) {
            return false;
        }

        if let Some(ref service_name) = self.service_filter {
            if entry.service != *service_name {
                return false;
            }
        }

        true
    }

    fn level_to_numeric(&self, level: &LogLevel) -> u8 {
        match level {
            LogLevel::DEBUG => 1,
            LogLevel::INFO => 2,
            LogLevel::WARN => 3,
            LogLevel::ERROR => 4,
            LogLevel::CRITICAL => 5,
        }
    }

    pub fn group_by_service(&self, entries: &[LogEntry]) -> HashMap<String, Vec<&LogEntry>> {
        let mut groups = HashMap::new();
        
        for entry in entries {
            groups
                .entry(entry.service.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }

        groups
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
        let mut counts = HashMap::new();
        
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(level: LogLevel, service: &str) -> LogEntry {
        LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level,
            service: service.to_string(),
            message: "Test message".to_string(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_level_filtering() {
        let processor = LogProcessor::new(LogLevel::WARN);
        let entries = vec![
            create_test_entry(LogLevel::INFO, "api"),
            create_test_entry(LogLevel::WARN, "api"),
            create_test_entry(LogLevel::ERROR, "api"),
        ];

        let filtered: Vec<&LogEntry> = entries
            .iter()
            .filter(|e| processor.should_process(e))
            .collect();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| {
            matches!(e.level, LogLevel::WARN | LogLevel::ERROR)
        }));
    }

    #[test]
    fn test_service_filtering() {
        let processor = LogProcessor::new(LogLevel::INFO).with_service_filter("auth");
        
        let entries = vec![
            create_test_entry(LogLevel::INFO, "api"),
            create_test_entry(LogLevel::INFO, "auth"),
            create_test_entry(LogLevel::ERROR, "database"),
        ];

        let filtered: Vec<&LogEntry> = entries
            .iter()
            .filter(|e| processor.should_process(e))
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].service, "auth");
    }
}