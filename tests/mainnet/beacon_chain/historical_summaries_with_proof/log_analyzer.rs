use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warning_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warning_pattern: Regex::new(r"WARNING").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_log_file(&self, file_path: &str) -> Result<LogSummary, String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary::new();
        
        for line in reader.lines() {
            let line_content = line.map_err(|e| e.to_string())?;
            self.process_line(&line_content, &mut summary);
        }
        
        Ok(summary)
    }

    fn process_line(&self, line: &str, summary: &mut LogSummary) {
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.categorize_error(line);
        } else if self.warning_pattern.is_match(line) {
            summary.warning_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
        
        summary.total_lines += 1;
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub error_categories: HashMap<String, usize>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            error_categories: HashMap::new(),
        }
    }

    pub fn categorize_error(&mut self, error_line: &str) {
        let category = if error_line.contains("database") {
            "database"
        } else if error_line.contains("network") {
            "network"
        } else if error_line.contains("authentication") {
            "authentication"
        } else {
            "other"
        };
        
        *self.error_categories.entry(category.to_string()).or_insert(0) += 1;
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        
        if !self.error_categories.is_empty() {
            println!("\nError Categories:");
            for (category, count) in &self.error_categories {
                println!("  {}: {}", category, count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert!(analyzer.error_pattern.is_match("ERROR: Something went wrong"));
        assert!(analyzer.warning_pattern.is_match("WARNING: Potential issue"));
        assert!(analyzer.info_pattern.is_match("INFO: System started"));
    }

    #[test]
    fn test_summary_categorization() {
        let mut summary = LogSummary::new();
        summary.categorize_error("ERROR: database connection failed");
        summary.categorize_error("ERROR: network timeout");
        summary.categorize_error("ERROR: authentication failed");
        
        assert_eq!(summary.error_categories.get("database"), Some(&1));
        assert_eq!(summary.error_categories.get("network"), Some(&1));
        assert_eq!(summary.error_categories.get("authentication"), Some(&1));
    }
}