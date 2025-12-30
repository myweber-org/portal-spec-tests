use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_patterns: HashMap<String, usize>,
    total_lines: usize,
    time_range: (String, String),
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_patterns: HashMap::new(),
            total_lines: 0,
            time_range: (String::new(), String::new()),
        }
    }

    pub fn analyze_file(&mut self, filepath: &str) -> Result<(), String> {
        let file = File::open(filepath)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        let error_regex = Regex::new(r"ERROR|FATAL|CRITICAL").unwrap();
        let timestamp_regex = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| format!("Line read error: {}", e))?;
            self.total_lines += 1;

            if let Some(timestamp) = timestamp_regex.find(&line) {
                let ts = timestamp.as_str().to_string();
                if line_num == 0 {
                    self.time_range.0 = ts.clone();
                }
                self.time_range.1 = ts;
            }

            if error_regex.is_match(&line) {
                let error_key = extract_error_type(&line);
                *self.error_patterns.entry(error_key).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total lines processed: {}\n", self.total_lines));
        report.push_str(&format!("Time range: {} - {}\n", self.time_range.0, self.time_range.1));
        report.push_str("Error distribution:\n");
        
        for (error, count) in &self.error_patterns {
            report.push_str(&format!("  {}: {} occurrences\n", error, count));
        }
        
        report
    }
}

fn extract_error_type(line: &str) -> String {
    let patterns = [
        ("Database", r"database|sql|connection"),
        ("Network", r"network|timeout|connection refused"),
        ("Authentication", r"auth|login|permission"),
        ("Memory", r"memory|out of memory|heap"),
    ];

    for (category, pattern) in patterns.iter() {
        let re = Regex::new(pattern).unwrap();
        if re.is_match(&line.to_lowercase()) {
            return category.to_string();
        }
    }
    
    "Unknown".to_string()
}