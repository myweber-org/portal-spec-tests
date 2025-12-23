use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    source_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            source_counts: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] \[(?P<level>\w+)\] \[(?P<source>[^\]]+)\] (?P<message>.+)").unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures["timestamp"].to_string(),
                    level: captures["level"].to_string(),
                    message: captures["message"].to_string(),
                    source: captures["source"].to_string(),
                };

                *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
                *self.source_counts.entry(entry.source.clone()).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn get_level_summary(&self) -> Vec<(&String, &usize)> {
        let mut summary: Vec<_> = self.level_counts.iter().collect();
        summary.sort_by(|a, b| b.1.cmp(a.1));
        summary
    }

    pub fn get_source_summary(&self) -> Vec<(&String, &usize)> {
        let mut summary: Vec<_> = self.source_counts.iter().collect();
        summary.sort_by(|a, b| b.1.cmp(a.1));
        summary
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn search_messages(&self, pattern: &str) -> Vec<&LogEntry> {
        let search_regex = Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap());
        self.entries.iter()
            .filter(|entry| search_regex.is_match(&entry.message))
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn error_rate(&self) -> f64 {
        let error_count = self.level_counts.get("ERROR").unwrap_or(&0);
        if self.entries.is_empty() {
            0.0
        } else {
            *error_count as f64 / self.entries.len() as f64 * 100.0
        }
    }
}