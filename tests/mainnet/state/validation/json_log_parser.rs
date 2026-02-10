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
            let levels = vec!["trace", "debug", "info", "warn", "error"];
            let entry_idx = levels.iter().position(|&l| l == entry.level.to_lowercase());
            let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());
            
            match (entry_idx, min_idx) {
                (Some(e), Some(m)) if e < m => return false,
                _ => {}
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

        true
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
            Ok(entry) if filter.matches(&entry) => entries.push(entry),
            Ok(_) => continue,
            Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
        }
    }

    Ok(entries)
}

fn analyze_logs(entries: &[LogEntry]) -> HashMap<String, usize> {
    let mut level_counts = HashMap::new();
    let mut message_patterns = HashMap::new();

    for entry in entries {
        *level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        
        let words: Vec<&str> = entry.message.split_whitespace().collect();
        if words.len() > 2 {
            let pattern = format!("{} {} ...", words[0], words[1]);
            *message_patterns.entry(pattern).or_insert(0) += 1;
        }
    }

    level_counts
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        start_time: Some(Utc::now() - chrono::Duration::hours(24)),
        end_time: Some(Utc::now()),
    };

    let entries = parse_log_file("application.log", &filter)?;
    let stats = analyze_logs(&entries);

    println!("Filtered entries: {}", entries.len());
    println!("Level statistics:");
    for (level, count) in stats {
        println!("  {}: {}", level, count);
    }

    if !entries.is_empty() {
        println!("\nSample entry:");
        println!("{:?}", entries[0]);
    }

    Ok(())
}