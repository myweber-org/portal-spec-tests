use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct LogFilter {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let levels = vec!["trace", "debug", "info", "warn", "error"];
            let entry_idx = levels.iter().position(|&l| l == entry.level.to_lowercase());
            let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());
            
            if let (Some(ei), Some(mi)) = (entry_idx, min_idx) {
                if ei < mi {
                    return false;
                }
            }
        }

        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if let Ok(entry_time) = entry.timestamp.parse::<DateTime<Utc>>() {
                if entry_time < *start || entry_time > *end {
                    return false;
                }
            }
        }

        true
    }
}

fn parse_log_file(path: &str, filter: &LogFilter) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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
                if filter.matches(&entry) {
                    entries.push(entry);
                }
            }
            Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
        }
    }

    Ok(entries)
}

fn main() {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        start_time: Some("2024-01-01T00:00:00Z".parse().unwrap()),
        end_time: Some("2024-12-31T23:59:59Z".parse().unwrap()),
    };

    match parse_log_file("app.log", &filter) {
        Ok(entries) => {
            println!("Found {} matching log entries:", entries.len());
            for entry in entries.iter().take(5) {
                println!("{:?}", entry);
            }
        }
        Err(e) => eprintln!("Error parsing log file: {}", e),
    }
}use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    component: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
struct LogParser {
    min_level: String,
    component_filter: Option<String>,
}

impl LogParser {
    fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            component_filter: None,
        }
    }

    fn with_component_filter(mut self, component: &str) -> Self {
        self.component_filter = Some(component.to_string());
        self
    }

    fn parse_file(&self, file_path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn Error>> {
        let entry: LogEntry = serde_json::from_str(line)?;
        
        if !self.matches_level(&entry.level) {
            return Err("Log level below threshold".into());
        }

        if let Some(ref filter) = self.component_filter {
            if let Some(ref component) = entry.component {
                if component != filter {
                    return Err("Component filter mismatch".into());
                }
            } else {
                return Err("No component specified".into());
            }
        }

        Ok(entry)
    }

    fn matches_level(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error", "fatal"];
        let entry_level = level.to_lowercase();
        
        if let Some(entry_idx) = level_order.iter().position(|&l| l == entry_level) {
            if let Some(min_idx) = level_order.iter().position(|&l| l == self.min_level) {
                return entry_idx >= min_idx;
            }
        }
        false
    }

    fn filter_by_timestamp_range(
        entries: &[LogEntry],
        start: &str,
        end: &str,
    ) -> Result<Vec<&LogEntry>, Box<dyn Error>> {
        let start_time: DateTime<Utc> = start.parse()?;
        let end_time: DateTime<Utc> = end.parse()?;
        
        let filtered: Vec<_> = entries
            .iter()
            .filter(|entry| {
                if let Ok(entry_time) = entry.timestamp.parse::<DateTime<Utc>>() {
                    entry_time >= start_time && entry_time <= end_time
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("info")
        .with_component_filter("api_server");

    let entries = parser.parse_file("logs/app.log")?;
    
    println!("Found {} log entries", entries.len());
    
    for entry in entries.iter().take(5) {
        println!("{:?}", entry);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_matching() {
        let parser = LogParser::new("warn");
        assert!(parser.matches_level("ERROR"));
        assert!(parser.matches_level("warn"));
        assert!(!parser.matches_level("info"));
    }

    #[test]
    fn test_component_filter() {
        let parser = LogParser::new("debug").with_component_filter("database");
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Query executed","component":"database"}"#;
        
        let result = parser.parse_line(log_line);
        assert!(result.is_ok());
    }
}