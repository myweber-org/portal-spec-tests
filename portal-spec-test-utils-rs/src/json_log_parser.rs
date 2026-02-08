use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

struct LogParser {
    min_level: String,
    filter_text: Option<String>,
}

impl LogParser {
    fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            filter_text: None,
        }
    }

    fn with_filter(mut self, filter: &str) -> Self {
        self.filter_text = Some(filter.to_lowercase());
        self
    }

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.should_include(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        let entry_level = entry.level.to_lowercase();
        let level_priority = |level: &str| match level {
            "error" => 3,
            "warn" => 2,
            "info" => 1,
            "debug" => 0,
            _ => 0,
        };

        if level_priority(&entry_level) < level_priority(&self.min_level) {
            return false;
        }

        if let Some(filter) = &self.filter_text {
            if !entry.message.to_lowercase().contains(filter) {
                return false;
            }
        }

        true
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("info")
        .with_filter("connection");

    let entries = parser.parse_file("app.log")?;
    
    for entry in entries {
        println!("[{}] {}: {}", 
            entry.timestamp, 
            entry.level.to_uppercase(), 
            entry.message
        );
    }

    Ok(())
}use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    component: String,
}

struct LogFilter {
    min_level: Option<LogLevel>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    component_filter: Option<String>,
}

impl LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            match (&entry.level, min_level) {
                (LogLevel::ERROR, _) => true,
                (LogLevel::WARN, LogLevel::WARN | LogLevel::INFO | LogLevel::DEBUG) => true,
                (LogLevel::INFO, LogLevel::INFO | LogLevel::DEBUG) => true,
                (LogLevel::DEBUG, LogLevel::DEBUG) => true,
                _ => false,
            }
        } else {
            true
        } && if let Some(start) = &self.start_time {
            &entry.timestamp >= start
        } else {
            true
        } && if let Some(end) = &self.end_time {
            &entry.timestamp <= end
        } else {
            true
        } && if let Some(component) = &self.component_filter {
            &entry.component == component
        } else {
            true
        }
    }
}

fn parse_log_file(path: &str, filter: &LogFilter) -> Result<Vec<LogEntry>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) if filter.matches(&entry) => entries.push(entry),
            Ok(_) => continue,
            Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
        }
    }

    Ok(entries)
}

fn analyze_logs(entries: &[LogEntry]) {
    let mut level_counts = std::collections::HashMap::new();
    let mut component_counts = std::collections::HashMap::new();

    for entry in entries {
        *level_counts.entry(&entry.level).or_insert(0) += 1;
        *component_counts.entry(&entry.component).or_insert(0) += 1;
    }

    println!("Log Analysis:");
    println!("Total entries: {}", entries.len());
    println!("\nBy level:");
    for (level, count) in &level_counts {
        println!("  {:?}: {}", level, count);
    }
    println!("\nBy component:");
    for (component, count) in &component_counts {
        println!("  {}: {}", component, count);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let filter = LogFilter {
        min_level: Some(LogLevel::INFO),
        start_time: Some(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?.with_timezone(&Utc)),
        end_time: Some(DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")?.with_timezone(&Utc)),
        component_filter: Some("api".to_string()),
    };

    let entries = parse_log_file("logs.jsonl", &filter)?;
    analyze_logs(&entries);

    if let Some(most_recent) = entries.iter().max_by_key(|e| e.timestamp) {
        println!("\nMost recent log:");
        println!("  Time: {}", most_recent.timestamp);
        println!("  Level: {:?}", most_recent.level);
        println!("  Component: {}", most_recent.component);
        println!("  Message: {}", most_recent.message);
    }

    Ok(())
}