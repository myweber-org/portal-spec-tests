use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    message: String,
    source: String,
}

impl LogEntry {
    fn new(timestamp: DateTime<FixedOffset>, level: &str, message: &str, source: &str) -> Self {
        LogEntry {
            timestamp,
            level: level.to_string(),
            message: message.to_string(),
            source: source.to_string(),
        }
    }
}

struct LogProcessor {
    entries: Vec<LogEntry>,
    filtered_entries: Vec<LogEntry>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            filtered_entries: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        self.filtered_entries = self.entries.clone();
        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        match DateTime::parse_from_rfc3339(parts[0].trim()) {
            Ok(timestamp) => Some(LogEntry::new(
                timestamp,
                parts[1].trim(),
                parts[2].trim(),
                parts[3].trim(),
            )),
            Err(_) => None,
        }
    }

    fn filter_by_level(&mut self, level: &str) {
        self.filtered_entries = self
            .entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .cloned()
            .collect();
    }

    fn filter_by_source(&mut self, source: &str) {
        self.filtered_entries = self
            .entries
            .iter()
            .filter(|entry| entry.source.contains(source))
            .cloned()
            .collect();
    }

    fn filter_by_time_range(
        &mut self,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    ) {
        self.filtered_entries = self
            .entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .cloned()
            .collect();
    }

    fn export_filtered<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;
        for entry in &self.filtered_entries {
            writeln!(
                file,
                "{} | {} | {} | {}",
                entry.timestamp.to_rfc3339(),
                entry.level,
                entry.message,
                entry.source
            )?;
        }
        Ok(())
    }

    fn get_statistics(&self) -> (usize, usize, Vec<(String, usize)>) {
        let total = self.entries.len();
        let filtered = self.filtered_entries.len();
        
        let mut level_counts = std::collections::HashMap::new();
        for entry in &self.filtered_entries {
            *level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        let mut level_vec: Vec<(String, usize)> = level_counts.into_iter().collect();
        level_vec.sort_by(|a, b| b.1.cmp(&a.1));
        
        (total, filtered, level_vec)
    }
}

fn main() -> io::Result<()> {
    let mut processor = LogProcessor::new();
    
    match processor.load_from_file("application.log") {
        Ok(_) => println!("Log file loaded successfully"),
        Err(e) => {
            eprintln!("Failed to load log file: {}", e);
            return Ok(());
        }
    }
    
    processor.filter_by_level("ERROR");
    
    let (total, filtered, stats) = processor.get_statistics();
    println!("Total entries: {}", total);
    println!("Filtered entries: {}", filtered);
    println!("Level distribution:");
    for (level, count) in stats {
        println!("  {}: {}", level, count);
    }
    
    processor.export_filtered("filtered_errors.log")?;
    println!("Filtered logs exported to filtered_errors.log");
    
    Ok(())
}