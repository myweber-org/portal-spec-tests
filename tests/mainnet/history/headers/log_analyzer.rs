use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    component: String,
    message: String,
}

#[derive(Debug)]
pub struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    component_counts: HashMap<String, usize>,
    level_distribution: HashMap<String, usize>,
}

pub struct LogAnalyzer {
    pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        let pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] (?P<level>\w+) \[(?P<component>[^\]]+)\]: (?P<message>.+)").unwrap();
        LogAnalyzer { pattern }
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        self.pattern.captures(line).and_then(|caps| {
            let timestamp_str = caps.name("timestamp")?.as_str();
            let level = caps.name("level")?.as_str().to_string();
            let component = caps.name("component")?.as_str().to_string();
            let message = caps.name("message")?.as_str().to_string();

            DateTime::parse_from_rfc3339(timestamp_str)
                .ok()
                .map(|timestamp| LogEntry {
                    timestamp,
                    level,
                    component,
                    message,
                })
        })
    }

    pub fn analyze_file(&self, path: &str) -> std::io::Result<LogSummary> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            component_counts: HashMap::new(),
            level_distribution: HashMap::new(),
        };

        for line_result in reader.lines() {
            let line = line_result?;
            if let Some(entry) = self.parse_line(&line) {
                summary.total_entries += 1;
                
                *summary.level_distribution.entry(entry.level.clone()).or_insert(0) += 1;
                *summary.component_counts.entry(entry.component.clone()).or_insert(0) += 1;
                
                match entry.level.as_str() {
                    "ERROR" => summary.error_count += 1,
                    "WARN" => summary.warning_count += 1,
                    _ => {}
                }
            }
        }

        Ok(summary)
    }

    pub fn find_errors(&self, path: &str) -> std::io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut errors = Vec::new();
        
        for line_result in reader.lines() {
            let line = line_result?;
            if let Some(entry) = self.parse_line(&line) {
                if entry.level == "ERROR" {
                    errors.push(entry);
                }
            }
        }
        
        Ok(errors)
    }
}

impl LogSummary {
    pub fn print_report(&self) {
        println!("Log Analysis Report");
        println!("===================");
        println!("Total entries: {}", self.total_entries);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("\nLevel Distribution:");
        for (level, count) in &self.level_distribution {
            println!("  {}: {}", level, count);
        }
        println!("\nComponent Activity:");
        for (component, count) in &self.component_counts {
            println!("  {}: {}", component, count);
        }
    }
}