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
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures[1].to_string(),
                    level: captures[2].to_string(),
                    message: captures[3].to_string(),
                };

                *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn get_level_summary(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn count_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
}

pub fn analyze_logs(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = LogAnalyzer::new();
    analyzer.parse_file(filepath)?;

    println!("Total log entries: {}", analyzer.count_entries());
    println!("Level distribution:");
    for (level, count) in analyzer.get_level_summary() {
        println!("  {}: {}", level, count);
    }

    let error_logs = analyzer.filter_by_level("ERROR");
    if !error_logs.is_empty() {
        println!("\nError entries found:");
        for entry in error_logs.iter().take(5) {
            println!("  [{}] {}", entry.timestamp, entry.message);
        }
    }

    Ok(())
}use std::collections::HashMap;
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
            warning_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            self.analyze_line(&line, &mut stats);
        }
        
        Ok(stats)
    }

    fn analyze_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("ERROR".to_string()).or_insert(0) += 1;
        } else if self.warning_pattern.is_match(line) {
            *stats.entry("WARN".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("INFO".to_string()).or_insert(0) += 1;
        }
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::new();
        report.push_str("Log Analysis Report\n");
        report.push_str("===================\n");
        
        for (level, count) in stats {
            report.push_str(&format!("{}: {}\n", level, count));
        }
        
        let total: usize = stats.values().sum();
        report.push_str(&format!("\nTotal log entries analyzed: {}", total));
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Low memory").unwrap();
        writeln!(temp_file, "ERROR: Connection failed").unwrap();
        writeln!(temp_file, "INFO: User logged in").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("INFO"), Some(&2));
        assert_eq!(stats.get("WARN"), Some(&1));
        assert_eq!(stats.get("ERROR"), Some(&1));
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub component: String,
    pub message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    component_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            component_counts: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.add_entry(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() == 4 {
            Some(LogEntry {
                timestamp: parts[0].trim().to_string(),
                level: parts[1].trim().to_string(),
                component: parts[2].trim().to_string(),
                message: parts[3].trim().to_string(),
            })
        } else {
            None
        }
    }

    fn add_entry(&mut self, entry: LogEntry) {
        *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        *self.component_counts.entry(entry.component.clone()).or_insert(0) += 1;
        self.entries.push(entry);
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_component(&self, component: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.component == component)
            .collect()
    }

    pub fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let error_count = self.level_counts.get("ERROR").unwrap_or(&0);
        let warning_count = self.level_counts.get("WARNING").unwrap_or(&0);

        format!(
            "Total entries: {}\nErrors: {}\nWarnings: {}\nUnique components: {}",
            total_entries,
            error_count,
            warning_count,
            self.component_counts.len()
        )
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.parse_line(&line);
        }

        Ok(())
    }

    fn parse_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() == 4 {
            let entry = LogEntry {
                timestamp: parts[0].trim().to_string(),
                level: parts[1].trim().to_string(),
                source: parts[2].trim().to_string(),
                message: parts[3].trim().to_string(),
            };

            *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
            *self.source_counts.entry(entry.source.clone()).or_insert(0) += 1;
            self.entries.push(entry);
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_source(&self, source: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.source == source)
            .collect()
    }

    pub fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let mut level_summary = String::new();
        
        for (level, count) in &self.level_counts {
            level_summary.push_str(&format!("{}: {} entries\n", level, count));
        }

        format!("Total log entries: {}\n{}", total_entries, level_summary)
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 10:00:00 | INFO | server | Application started").unwrap();
        writeln!(temp_file, "2023-10-01 10:05:00 | ERROR | database | Connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 10:10:00 | WARN | server | High memory usage detected").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.filter_by_source("server").len(), 2);
        
        let summary = analyzer.get_summary();
        assert!(summary.contains("Total log entries: 3"));
        assert!(summary.contains("INFO: 1 entries"));
        assert!(summary.contains("ERROR: 1 entries"));
        assert!(summary.contains("WARN: 1 entries"));
    }
}