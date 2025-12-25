use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    module: Option<String>,
    thread_id: Option<u32>,
}

struct LogFilter {
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    module_filter: Option<String>,
}

impl LogFilter {
    fn new(min_level: LogLevel) -> Self {
        LogFilter {
            min_level,
            start_time: None,
            end_time: None,
            module_filter: None,
        }
    }

    fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    fn with_module_filter(mut self, module: &str) -> Self {
        self.module_filter = Some(module.to_string());
        self
    }

    fn matches(&self, entry: &LogEntry) -> bool {
        if entry.level as u8 > self.min_level as u8 {
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

        if let Some(ref module_filter) = self.module_filter {
            if let Some(ref module) = entry.module {
                if module != module_filter {
                    return false;
                }
            } else {
                return false;
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

fn analyze_logs(entries: &[LogEntry]) {
    let mut level_counts = std::collections::HashMap::new();
    let mut module_counts = std::collections::HashMap::new();

    for entry in entries {
        *level_counts.entry(&entry.level).or_insert(0) += 1;
        if let Some(module) = &entry.module {
            *module_counts.entry(module).or_insert(0) += 1;
        }
    }

    println!("Log Analysis:");
    println!("Total entries: {}", entries.len());
    println!("\nBy level:");
    for (level, count) in &level_counts {
        println!("  {:?}: {}", level, count);
    }
    println!("\nBy module:");
    for (module, count) in &module_counts {
        println!("  {}: {}", module, count);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter::new(LogLevel::INFO)
        .with_time_range(
            DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?.with_timezone(&Utc),
            DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")?.with_timezone(&Utc),
        )
        .with_module_filter("database");

    let entries = parse_log_file("logs/app.log", &filter)?;
    
    if entries.is_empty() {
        println!("No log entries match the filter criteria.");
    } else {
        analyze_logs(&entries);
        
        println!("\nSample entries:");
        for entry in entries.iter().take(3) {
            println!("{:?}", entry);
        }
    }

    Ok(())
}