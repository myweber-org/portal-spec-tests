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

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    fn parse_log_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp = captures[1].to_string();
                let level = captures[2].to_string();
                let message = captures[3].to_string();

                let entry = LogEntry {
                    timestamp,
                    level: level.clone(),
                    message,
                };

                self.entries.push(entry);
                *self.level_counts.entry(level).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    fn generate_summary(&self) {
        println!("Log Analysis Summary");
        println!("====================");
        println!("Total entries: {}", self.entries.len());
        println!("\nLog level distribution:");

        for (level, count) in &self.level_counts {
            let percentage = (*count as f64 / self.entries.len() as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", level, count, percentage);
        }

        if let Some(error_count) = self.level_counts.get("ERROR") {
            if *error_count > 0 {
                println!("\nError entries found:");
                for entry in &self.entries {
                    if entry.level == "ERROR" {
                        println!("  [{}] {}", entry.timestamp, entry.message);
                    }
                }
            }
        }
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
}

fn main() {
    let mut analyzer = LogAnalyzer::new();

    match analyzer.parse_log_file("application.log") {
        Ok(_) => {
            analyzer.generate_summary();
            
            let warnings = analyzer.filter_by_level("WARN");
            if !warnings.is_empty() {
                println!("\nWarning entries ({} total):", warnings.len());
                for entry in warnings.iter().take(5) {
                    println!("  [{}] {}", entry.timestamp, entry.message);
                }
            }
        }
        Err(e) => eprintln!("Failed to parse log file: {}", e),
    }
}