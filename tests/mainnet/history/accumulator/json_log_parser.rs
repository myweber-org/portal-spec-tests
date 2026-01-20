use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct LogFilter {
    pub min_level: Option<LogLevel>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub component_filter: Option<String>,
}

impl LogFilter {
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            if !self.compare_levels(&entry.level, min_level) {
                return false;
            }
        }

        if let Some(start) = &self.start_time {
            if &entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = &self.end_time {
            if &entry.timestamp > end {
                return false;
            }
        }

        if let Some(component) = &self.component_filter {
            if !entry.component.contains(component) {
                return false;
            }
        }

        true
    }

    fn compare_levels(&self, entry_level: &LogLevel, filter_level: &LogLevel) -> bool {
        let level_value = |level: &LogLevel| match level {
            LogLevel::ERROR => 4,
            LogLevel::WARN => 3,
            LogLevel::INFO => 2,
            LogLevel::DEBUG => 1,
            LogLevel::TRACE => 0,
        };

        level_value(entry_level) >= level_value(filter_level)
    }
}

pub struct LogParser {
    filter: LogFilter,
}

impl LogParser {
    pub fn new(filter: LogFilter) -> Self {
        LogParser { filter }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    if self.filter.matches(&entry) {
                        entries.push(entry);
                    }
                }
                Err(e) => eprintln!("Failed to parse log line: {}", e),
            }
        }

        Ok(entries)
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, entries: &[LogEntry], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(entries)?;
        std::fs::write(output_path, json)?;
        Ok(())
    }

    pub fn statistics(&self, entries: &[LogEntry]) -> std::collections::HashMap<LogLevel, usize> {
        let mut stats = std::collections::HashMap::new();
        
        for entry in entries {
            *stats.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_filter_matching() {
        let entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            level: LogLevel::INFO,
            message: "Test message".to_string(),
            component: "api".to_string(),
            metadata: None,
        };

        let filter = LogFilter {
            min_level: Some(LogLevel::INFO),
            start_time: Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()),
            end_time: Some(Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap()),
            component_filter: Some("api".to_string()),
        };

        assert!(filter.matches(&entry));
    }

    #[test]
    fn test_level_comparison() {
        let filter = LogFilter {
            min_level: Some(LogLevel::WARN),
            start_time: None,
            end_time: None,
            component_filter: None,
        };

        assert!(filter.compare_levels(&LogLevel::ERROR, &LogLevel::WARN));
        assert!(filter.compare_levels(&LogLevel::WARN, &LogLevel::WARN));
        assert!(!filter.compare_levels(&LogLevel::INFO, &LogLevel::WARN));
    }
}