use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    error: Option<String>,
}

fn parse_log_file(file_path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
        }
    }

    Ok(entries)
}

fn filter_errors(entries: Vec<LogEntry>) -> Vec<LogEntry> {
    entries
        .into_iter()
        .filter(|entry| entry.level == "ERROR")
        .collect()
}

fn format_timestamp(entry: &LogEntry) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&entry.timestamp)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn main() {
    let file_path = "app.log";
    
    match parse_log_file(file_path) {
        Ok(entries) => {
            let error_entries = filter_errors(entries);
            
            println!("Found {} error entries:", error_entries.len());
            for entry in error_entries {
                println!("[{}] {}", entry.level, entry.message);
                if let Some(error) = &entry.error {
                    println!("  Error details: {}", error);
                }
                if let Some(dt) = format_timestamp(&entry) {
                    println!("  Timestamp: {}", dt.format("%Y-%m-%d %H:%M:%S"));
                }
            }
        }
        Err(e) => eprintln!("Failed to parse log file: {}", e),
    }
}