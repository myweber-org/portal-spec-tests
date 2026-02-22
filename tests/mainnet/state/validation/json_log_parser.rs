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
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

struct LogParser {
    min_level: Option<String>,
    search_term: Option<String>,
    time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            min_level: None,
            search_term: None,
            time_range: None,
        }
    }

    fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    fn with_search(mut self, term: &str) -> Self {
        self.search_term = Some(term.to_lowercase());
        self
    }

    fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.time_range = Some((start, end));
        self
    }

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.filter_entry(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn filter_entry(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let entry_level = entry.level.to_lowercase();
            let level_order = ["trace", "debug", "info", "warn", "error"];
            let entry_idx = level_order.iter().position(|&l| l == entry_level);
            let min_idx = level_order.iter().position(|&l| l == min_level.as_str());
            
            if let (Some(ei), Some(mi)) = (entry_idx, min_idx) {
                if ei < mi {
                    return false;
                }
            }
        }

        if let Some(term) = &self.search_term {
            if !entry.message.to_lowercase().contains(term) {
                return false;
            }
        }

        if let Some((start, end)) = &self.time_range {
            if entry.timestamp < *start || entry.timestamp > *end {
                return false;
            }
        }

        true
    }

    fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!(
            "[{}] {}: {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.level.to_uppercase(),
            entry.message
        );

        if !entry.extra.is_empty() {
            output.push_str(&format!(" | Extra: {:?}", entry.extra));
        }

        output
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = LogParser::new()
        .with_min_level("info")
        .with_search("connection");

    let entries = parser.parse_file("logs/app.log")?;
    
    for entry in entries {
        println!("{}", parser.format_entry(&entry));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_parser_filtering() {
        let parser = LogParser::new()
            .with_min_level("warn")
            .with_time_range(
                Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
            );

        let test_entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 6, 15, 10, 30, 0).unwrap(),
            level: "error".to_string(),
            message: "Database connection failed".to_string(),
            extra: HashMap::new(),
        };

        assert!(parser.filter_entry(&test_entry));
    }
}