use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    component: Option<String>,
}

#[derive(Debug)]
struct LogFilter {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    component_filter: Option<String>,
}

impl LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let levels = ["trace", "debug", "info", "warn", "error"];
            let entry_idx = levels.iter().position(|&l| l == entry.level.to_lowercase());
            let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());
            
            match (entry_idx, min_idx) {
                (Some(e), Some(m)) if e < m => return false,
                _ => (),
            }
        }

        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if let Ok(entry_time) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                let entry_utc = entry_time.with_timezone(&Utc);
                if entry_utc < *start || entry_utc > *end {
                    return false;
                }
            }
        }

        if let Some(component) = &self.component_filter {
            if let Some(entry_component) = &entry.component {
                if !entry_component.contains(component) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
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

fn main() -> Result<(), Box<dyn Error>> {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        start_time: Some(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?.with_timezone(&Utc)),
        end_time: Some(DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")?.with_timezone(&Utc)),
        component_filter: Some("api".to_string()),
    };

    let entries = parse_log_file("application.log", &filter)?;
    
    println!("Found {} matching log entries", entries.len());
    for entry in entries.iter().take(5) {
        println!("{:?}", entry);
    }

    Ok(())
}