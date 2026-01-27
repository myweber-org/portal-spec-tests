use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warn_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warn_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_file(&self, file_path: &str) -> Result<HashMap<String, usize>, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();

        stats.insert("total_lines".to_string(), 0);
        stats.insert("errors".to_string(), 0);
        stats.insert("warnings".to_string(), 0);
        stats.insert("info".to_string(), 0);

        for line in reader.lines() {
            let line = line?;
            *stats.get_mut("total_lines").unwrap() += 1;

            if self.error_pattern.is_match(&line) {
                *stats.get_mut("errors").unwrap() += 1;
            } else if self.warn_pattern.is_match(&line) {
                *stats.get_mut("warnings").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.get_mut("info").unwrap() += 1;
            }
        }

        Ok(stats)
    }

    pub fn print_summary(&self, stats: &HashMap<String, usize>) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", stats.get("total_lines").unwrap_or(&0));
        println!("Errors: {}", stats.get("errors").unwrap_or(&0));
        println!("Warnings: {}", stats.get("warnings").unwrap_or(&0));
        println!("Info messages: {}", stats.get("info").unwrap_or(&0));
    }
}

pub fn process_logs(file_path: &str) {
    let analyzer = LogAnalyzer::new();
    match analyzer.analyze_file(file_path) {
        Ok(stats) => analyzer.print_summary(&stats),
        Err(e) => eprintln!("Failed to analyze log file: {}", e),
    }
}