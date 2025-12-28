use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
            if !self.level_matches(&entry.level, min_level) {
                return false;
            }
        }

        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if let Ok(entry_time) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                let utc_time = entry_time.with_timezone(&Utc);
                if utc_time < *start || utc_time > *end {
                    return false;
                }
            }
        }

        true
    }

    fn level_matches(&self, entry_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error"];
        let entry_idx = levels.iter().position(|&l| l == entry_level.to_lowercase());
        let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());

        match (entry_idx, min_idx) {
            (Some(e), Some(m)) => e >= m,
            _ => false,
        }
    }
}

fn parse_log_file<P: AsRef<Path>>(path: P, filter: &LogFilter) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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
            Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
        }
    }

    Ok(entries)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        start_time: Some(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?.with_timezone(&Utc)),
        end_time: Some(DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")?.with_timezone(&Utc)),
    };

    let entries = parse_log_file("application.log", &filter)?;
    
    println!("Found {} log entries matching criteria:", entries.len());
    for entry in entries.iter().take(5) {
        println!("[{:?}] {}: {}", entry.level, entry.timestamp, entry.message);
    }

    Ok(())
}