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

pub struct LogParser {
    file_path: String,
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
            min_level: LogLevel::INFO,
            start_time: None,
            end_time: None,
        }
    }

    pub fn set_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    pub fn set_time_range(mut self, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        self.start_time = start;
        self.end_time = end;
        self
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            
            if !self.filter_entry(&entry) {
                continue;
            }

            entries.push(entry);
        }

        Ok(entries)
    }

    fn filter_entry(&self, entry: &LogEntry) -> bool {
        if entry.level < self.min_level {
            return false;
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        true
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_val = match self {
            LogLevel::ERROR => 4,
            LogLevel::WARN => 3,
            LogLevel::INFO => 2,
            LogLevel::DEBUG => 1,
            LogLevel::TRACE => 0,
        };
        let other_val = match other {
            LogLevel::ERROR => 4,
            LogLevel::WARN => 3,
            LogLevel::INFO => 2,
            LogLevel::DEBUG => 1,
            LogLevel::TRACE => 0,
        };
        self_val.partial_cmp(&other_val)
    }
}