use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    module: String,
    message: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct LogProcessor {
    entries: Vec<LogEntry>,
    stats: HashMap<String, usize>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            stats: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    self.update_stats(&entry);
                    self.entries.push(entry);
                }
                Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
            }
        }

        Ok(())
    }

    fn update_stats(&mut self, entry: &LogEntry) {
        *self.stats.entry(entry.level.clone()).or_insert(0) += 1;
        *self.stats.entry(entry.module.clone()).or_insert(0) += 1;
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn filter_by_module(&self, module: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.module.contains(module))
            .collect()
    }

    pub fn get_statistics(&self) -> &HashMap<String, usize> {
        &self.stats
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, path: P, filtered_entries: Vec<&LogEntry>) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let entries: Vec<&LogEntry> = if filtered_entries.is_empty() {
            self.entries.iter().collect()
        } else {
            filtered_entries
        };
        
        serde_json::to_writer_pretty(file, &entries)?;
        Ok(())
    }
}

pub fn process_log_file(input_path: &str, output_path: &str, level_filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LogProcessor::new();
    processor.load_from_file(input_path)?;

    let filtered = if let Some(level) = level_filter {
        processor.filter_by_level(level)
    } else {
        Vec::new()
    };

    processor.export_to_json(output_path, filtered)?;

    println!("Processing complete.");
    println!("Total entries: {}", processor.entries.len());
    println!("Statistics: {:?}", processor.get_statistics());

    Ok(())
}