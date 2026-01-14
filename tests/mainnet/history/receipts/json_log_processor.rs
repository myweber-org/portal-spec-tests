use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: HashMap<String, String>,
}

struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: LogEntry = serde_json::from_str(&line)?;
            self.entries.push(entry);
        }
        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    fn group_by_service(&self) -> HashMap<String, Vec<&LogEntry>> {
        let mut groups = HashMap::new();
        for entry in &self.entries {
            groups
                .entry(entry.service.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        groups
    }

    fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LogProcessor::new();
    processor.load_from_file("logs.jsonl")?;

    println!("Total entries: {}", processor.entries.len());

    let error_logs = processor.filter_by_level("error");
    println!("Error logs: {}", error_logs.len());

    let service_groups = processor.group_by_service();
    for (service, logs) in service_groups {
        println!("Service '{}': {} logs", service, logs.len());
    }

    let level_counts = processor.count_by_level();
    for (level, count) in level_counts {
        println!("Level '{}': {} entries", level, count);
    }

    let search_results = processor.search_messages("timeout");
    println!("Found {} logs containing 'timeout'", search_results.len());

    Ok(())
}