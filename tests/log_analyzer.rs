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
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum LogSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogSeverity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Some(LogSeverity::Debug),
            "info" => Some(LogSeverity::Info),
            "warning" => Some(LogSeverity::Warning),
            "error" => Some(LogSeverity::Error),
            "critical" => Some(LogSeverity::Critical),
            _ => None,
        }
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub severity: LogSeverity,
    pub component: String,
    pub message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let severity = LogSeverity::from_str(parts[1].trim())?;

        Some(LogEntry {
            timestamp: parts[0].trim().to_string(),
            severity,
            component: parts[2].trim().to_string(),
            message: parts[3].trim().to_string(),
        })
    }

    pub fn filter_by_severity(&self, severity: LogSeverity) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.severity == severity)
            .collect()
    }

    pub fn count_by_severity(&self) -> Vec<(LogSeverity, usize)> {
        let mut counts = vec![
            (LogSeverity::Debug, 0),
            (LogSeverity::Info, 0),
            (LogSeverity::Warning, 0),
            (LogSeverity::Error, 0),
            (LogSeverity::Critical, 0),
        ];

        for entry in &self.entries {
            for (severity, count) in &mut counts {
                if entry.severity == *severity {
                    *count += 1;
                    break;
                }
            }
        }

        counts
    }

    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_parsing() {
        assert_eq!(LogSeverity::from_str("ERROR"), Some(LogSeverity::Error));
        assert_eq!(LogSeverity::from_str("warning"), Some(LogSeverity::Warning));
        assert_eq!(LogSeverity::from_str("unknown"), None);
    }

    #[test]
    fn test_log_parsing() {
        let analyzer = LogAnalyzer::new();
        let line = "2023-10-05 14:30:00 | ERROR | network | Connection timeout";
        
        let entry = analyzer.parse_log_line(line).unwrap();
        assert_eq!(entry.timestamp, "2023-10-05 14:30:00");
        assert_eq!(entry.severity, LogSeverity::Error);
        assert_eq!(entry.component, "network");
        assert_eq!(entry.message, "Connection timeout");
    }
}use std::collections::HashMap;
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

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line, &mut stats);
        }

        Ok(stats)
    }

    fn process_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("errors".to_string()).or_insert(0) += 1;
        } else if self.warn_pattern.is_match(line) {
            *stats.entry("warnings".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("info".to_string()).or_insert(0) += 1;
        }
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::from("Log Analysis Report\n");
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

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Database connection failed\nWARN: High memory usage\nINFO: Request processed";
        
        let mut stats = HashMap::new();
        for line in test_log.lines() {
            analyzer.process_line(line, &mut stats);
        }

        assert_eq!(stats.get("info"), Some(&2));
        assert_eq!(stats.get("errors"), Some(&1));
        assert_eq!(stats.get("warnings"), Some(&1));
    }
}use std::collections::HashMap;
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

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line, &mut stats);
        }

        Ok(stats)
    }

    fn process_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("errors".to_string()).or_insert(0) += 1;
        } else if self.warn_pattern.is_match(line) {
            *stats.entry("warnings".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("info".to_string()).or_insert(0) += 1;
        }
    }

    pub fn print_summary(&self, stats: &HashMap<String, usize>) {
        println!("Log Analysis Summary:");
        for (level, count) in stats {
            println!("  {}: {}", level, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Failed to connect\nWARN: Retrying connection\nINFO: Connection established";
        
        let mut stats = HashMap::new();
        for line in test_log.lines() {
            analyzer.process_line(line, &mut stats);
        }

        assert_eq!(stats.get("info"), Some(&2));
        assert_eq!(stats.get("errors"), Some(&1));
        assert_eq!(stats.get("warnings"), Some(&1));
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    module: String,
    message: String,
    metadata: HashMap<String, String>,
}

impl LogEntry {
    pub fn new(timestamp: DateTime<FixedOffset>, level: &str, module: &str, message: &str) -> Self {
        LogEntry {
            timestamp,
            level: level.to_string(),
            module: module.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn matches_filter(&self, level_filter: Option<&str>, module_filter: Option<&str>) -> bool {
        if let Some(level) = level_filter {
            if !self.level.eq_ignore_ascii_case(level) {
                return false;
            }
        }

        if let Some(module) = module_filter {
            if !self.module.contains(module) {
                return false;
            }
        }

        true
    }
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
                count += 1;
            }
        }

        Ok(count)
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 4 {
            return None;
        }

        let timestamp_str = format!("{} {}", parts[0], parts[1]);
        let level = parts[2];
        let module_message = parts[3];
        
        let module_message_parts: Vec<&str> = module_message.splitn(2, ':').collect();
        if module_message_parts.len() < 2 {
            return None;
        }

        let module = module_message_parts[0].trim();
        let message = module_message_parts[1].trim();

        match DateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S %z") {
            Ok(timestamp) => Some(LogEntry::new(timestamp, level, module, message)),
            Err(_) => None,
        }
    }

    pub fn filter_entries(&self, level: Option<&str>, module: Option<&str>) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.matches_filter(level, module))
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn get_entries_in_time_range(
        &self,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    ) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_entry_creation() {
        let timestamp = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2024, 1, 15, 10, 30, 0)
            .unwrap();
        
        let entry = LogEntry::new(timestamp, "INFO", "network", "Connection established");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.module, "network");
        assert_eq!(entry.message, "Connection established");
    }

    #[test]
    fn test_filter_matching() {
        let timestamp = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2024, 1, 15, 10, 30, 0)
            .unwrap();
        
        let entry = LogEntry::new(timestamp, "ERROR", "database", "Query failed");
        
        assert!(entry.matches_filter(Some("ERROR"), Some("data")));
        assert!(!entry.matches_filter(Some("WARN"), Some("data")));
        assert!(entry.matches_filter(None, Some("base")));
    }
}