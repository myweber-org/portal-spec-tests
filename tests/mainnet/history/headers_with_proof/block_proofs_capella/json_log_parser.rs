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
}