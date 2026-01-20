use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

#[derive(Debug)]
struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    unique_messages: HashMap<String, usize>,
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_messages: HashMap::new(),
        }
    }

    fn add_entry(&mut self, entry: &LogEntry) {
        self.total_entries += 1;
        match entry.level.as_str() {
            "ERROR" => self.error_count += 1,
            "WARN" => self.warning_count += 1,
            "INFO" => self.info_count += 1,
            _ => {}
        }
        *self.unique_messages.entry(entry.message.clone()).or_insert(0) += 1;
    }

    fn display(&self) {
        println!("Log Analysis Summary:");
        println!("Total entries: {}", self.total_entries);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        println!("\nUnique messages (count):");
        for (message, count) in &self.unique_messages {
            println!("  {}: {}", message, count);
        }
    }
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let re = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)").ok()?;
    let caps = re.captures(line)?;
    
    Some(LogEntry {
        timestamp: caps[1].to_string(),
        level: caps[2].to_string(),
        message: caps[3].to_string(),
    })
}

fn analyze_log_file(file_path: &str) -> Result<LogSummary, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut summary = LogSummary::new();

    for line in reader.lines() {
        let line = line?;
        if let Some(entry) = parse_log_line(&line) {
            summary.add_entry(&entry);
        }
    }

    Ok(summary)
}

fn main() {
    let file_path = "application.log";
    match analyze_log_file(file_path) {
        Ok(summary) => summary.display(),
        Err(e) => eprintln!("Error analyzing log file: {}", e),
    }
}