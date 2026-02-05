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
            error_pattern: Regex::new(r"(?i)error").unwrap(),
            warn_pattern: Regex::new(r"(?i)warn").unwrap(),
            info_pattern: Regex::new(r"(?i)info").unwrap(),
        }
    }

    pub fn analyze_file(&self, path: &str) -> Result<LogSummary, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            self.process_line(&line, &mut summary);
        }
        
        Ok(summary)
    }
    
    fn process_line(&self, line: &str, summary: &mut LogSummary) {
        summary.total_lines += 1;
        
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.error_lines.push(line.to_string());
        } else if self.warn_pattern.is_match(line) {
            summary.warn_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
        
        if line.contains("HTTP") {
            summary.http_requests += 1;
        }
    }
    
    pub fn get_top_errors(&self, summary: &LogSummary, limit: usize) -> Vec<String> {
        let mut error_map: HashMap<String, usize> = HashMap::new();
        
        for error_line in &summary.error_lines {
            let words: Vec<&str> = error_line.split_whitespace().collect();
            if words.len() > 2 {
                let key = format!("{} {}", words[0], words[1]);
                *error_map.entry(key).or_insert(0) += 1;
            }
        }
        
        let mut sorted_errors: Vec<(String, usize)> = error_map.into_iter().collect();
        sorted_errors.sort_by(|a, b| b.1.cmp(&a.1));
        
        sorted_errors
            .iter()
            .take(limit)
            .map(|(error, count)| format!("{}: {}", error, count))
            .collect()
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub http_requests: usize,
    pub error_lines: Vec<String>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            http_requests: 0,
            error_lines: Vec::new(),
        }
    }
    
    pub fn error_rate(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.total_lines as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_analyzer() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Database connection failed\nWARN: High memory usage\nINFO: User login successful\nERROR: File not found";
        
        let mut summary = LogSummary::new();
        for line in test_log.lines() {
            analyzer.process_line(line, &mut summary);
        }
        
        assert_eq!(summary.total_lines, 5);
        assert_eq!(summary.error_count, 2);
        assert_eq!(summary.warn_count, 1);
        assert_eq!(summary.info_count, 2);
        assert_eq!(summary.error_lines.len(), 2);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    component: String,
    message: String,
    metadata: HashMap<String, String>,
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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] (?P<level>\w+) (?P<component>[^:]+): (?P<message>.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(caps) = log_pattern.captures(&line) {
                let timestamp_str = caps.name("timestamp").unwrap().as_str();
                let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;
                
                let entry = LogEntry {
                    timestamp,
                    level: caps.name("level").unwrap().as_str().to_string(),
                    component: caps.name("component").unwrap().as_str().to_string(),
                    message: caps.name("message").unwrap().as_str().to_string(),
                    metadata: HashMap::new(),
                };
                
                self.entries.push(entry);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn filter_by_component(&self, component: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.component.contains(component))
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

    pub fn search_messages(&self, pattern: &str) -> Vec<&LogEntry> {
        let search_regex = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
        self.entries
            .iter()
            .filter(|entry| search_regex.is_match(&entry.message))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert_eq!(analyzer.entries.len(), 0);
    }

    #[test]
    fn test_filter_by_level() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: FixedOffset::east(0).ymd(2023, 1, 1).and_hms(0, 0, 0),
            level: "ERROR".to_string(),
            component: "database".to_string(),
            message: "Connection failed".to_string(),
            metadata: HashMap::new(),
        });
        
        analyzer.entries.push(LogEntry {
            timestamp: FixedOffset::east(0).ymd(2023, 1, 1).and_hms(0, 0, 1),
            level: "INFO".to_string(),
            component: "server".to_string(),
            message: "Server started".to_string(),
            metadata: HashMap::new(),
        });

        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)")?;

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

                *self.level_counts.entry(level).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn get_level_summary(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn contains_error(&self) -> bool {
        self.level_counts.contains_key("ERROR")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2024-01-15 10:30:00 [INFO] Application started").unwrap();
        writeln!(temp_file, "2024-01-15 10:31:00 [WARN] High memory usage detected").unwrap();
        writeln!(temp_file, "2024-01-15 10:32:00 [ERROR] Failed to connect to database").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.total_entries(), 3);
        assert!(analyzer.contains_error());
        
        let summary = analyzer.get_level_summary();
        assert_eq!(summary.get("INFO"), Some(&1));
        assert_eq!(summary.get("ERROR"), Some(&1));
    }
}