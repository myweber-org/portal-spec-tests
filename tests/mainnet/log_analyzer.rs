use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: String,
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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] \[(?P<level>\w+)\] \[(?P<source>[^\]]+)\] (?P<message>.+)").unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures["timestamp"].to_string(),
                    level: captures["level"].to_string(),
                    message: captures["message"].to_string(),
                    source: captures["source"].to_string(),
                };

                *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
                *self.source_counts.entry(entry.source.clone()).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn get_level_summary(&self) -> Vec<(&String, &usize)> {
        let mut summary: Vec<_> = self.level_counts.iter().collect();
        summary.sort_by(|a, b| b.1.cmp(a.1));
        summary
    }

    pub fn get_source_summary(&self) -> Vec<(&String, &usize)> {
        let mut summary: Vec<_> = self.source_counts.iter().collect();
        summary.sort_by(|a, b| b.1.cmp(a.1));
        summary
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn search_messages(&self, pattern: &str) -> Vec<&LogEntry> {
        let search_regex = Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap());
        self.entries.iter()
            .filter(|entry| search_regex.is_match(&entry.message))
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn error_rate(&self) -> f64 {
        let error_count = self.level_counts.get("ERROR").unwrap_or(&0);
        if self.entries.is_empty() {
            0.0
        } else {
            *error_count as f64 / self.entries.len() as f64 * 100.0
        }
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        self.update_statistics();
        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
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

    fn update_statistics(&mut self) {
        self.level_counts.clear();
        for entry in &self.entries {
            *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_summary(&self) -> HashMap<String, usize> {
        self.level_counts.clone()
    }

    pub fn count_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn find_messages_containing(&self, keyword: &str) -> Vec<&LogEntry> {
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
        writeln!(temp_file, "2023-10-01T10:00:00 INFO Application started").unwrap();
        writeln!(temp_file, "2023-10-01T10:01:00 ERROR Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01T10:02:00 WARN High memory usage detected").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.count_entries(), 3);
        
        let summary = analyzer.get_summary();
        assert_eq!(summary.get("INFO"), Some(&1));
        assert_eq!(summary.get("ERROR"), Some(&1));
        
        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Database connection failed");
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

                self.entries.push(entry);
                *self.level_counts.entry(level).or_insert(0) += 1;
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
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:30:45 [INFO] Application started"
        ).unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:31:22 [ERROR] Failed to connect to database"
        ).unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:32:10 [WARN] High memory usage detected"
        ).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.total_entries(), 3);
        assert!(analyzer.contains_error());
        
        let summary = analyzer.get_level_summary();
        assert_eq!(summary.get("INFO"), Some(&1));
        assert_eq!(summary.get("ERROR"), Some(&1));
        assert_eq!(summary.get("WARN"), Some(&1));
    }
}