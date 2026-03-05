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
            self.process_line(&line, &mut stats);
        }
        
        Ok(stats)
    }

    fn process_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("ERROR".to_string()).or_insert(0) += 1;
        } else if self.warning_pattern.is_match(line) {
            *stats.entry("WARN".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("INFO".to_string()).or_insert(0) += 1;
        }
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let total: usize = stats.values().sum();
        let mut report = format!("Log Analysis Report\n");
        report.push_str(&format!("Total log entries: {}\n", total));
        
        for (level, count) in stats {
            let percentage = (*count as f64 / total as f64) * 100.0;
            report.push_str(&format!("{}: {} ({:.1}%)\n", level, count, percentage));
        }
        
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
        writeln!(temp_file, "WARN: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: User login successful").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        let report = analyzer.generate_report(&stats);
        
        assert_eq!(stats.get("INFO"), Some(&2));
        assert_eq!(stats.get("WARN"), Some(&1));
        assert_eq!(stats.get("ERROR"), Some(&1));
        assert!(report.contains("Total log entries: 4"));
    }
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
            self.process_line(&line, &mut stats);
        }
        
        Ok(stats)
    }

    fn process_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("errors".to_string()).or_insert(0) += 1;
        } else if self.warning_pattern.is_match(line) {
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Connection failed").unwrap();
        writeln!(temp_file, "INFO: Processing complete").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("info"), Some(&2));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("errors"), Some(&1));
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

#[derive(Debug)]
pub struct LogStats {
    total_entries: usize,
    level_counts: HashMap<String, usize>,
    error_messages: Vec<String>,
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = Self::parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() == 3 {
            Some(LogEntry {
                timestamp: parts[0].to_string(),
                level: parts[1].to_string(),
                message: parts[2].to_string(),
            })
        } else {
            None
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_stats(&self) -> LogStats {
        let mut level_counts = HashMap::new();
        let mut error_messages = Vec::new();

        for entry in &self.entries {
            *level_counts.entry(entry.level.clone()).or_insert(0) += 1;

            if entry.level == "ERROR" {
                error_messages.push(entry.message.clone());
            }
        }

        LogStats {
            total_entries: self.entries.len(),
            level_counts,
            error_messages,
        }
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

    #[test]
    fn test_parse_log_line() {
        let line = "2023-10-01T12:00:00 INFO Application started";
        let entry = LogAnalyzer::parse_log_line(line).unwrap();

        assert_eq!(entry.timestamp, "2023-10-01T12:00:00");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Application started");
    }

    #[test]
    fn test_filter_by_level() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: "2023-10-01T12:00:00".to_string(),
            level: "INFO".to_string(),
            message: "Test message".to_string(),
        });
        analyzer.entries.push(LogEntry {
            timestamp: "2023-10-01T12:01:00".to_string(),
            level: "ERROR".to_string(),
            message: "Error occurred".to_string(),
        });

        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Error occurred");
    }
}