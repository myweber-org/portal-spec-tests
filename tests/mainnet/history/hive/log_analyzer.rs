use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "INFO" => Some(LogLevel::INFO),
            "WARN" => Some(LogLevel::WARN),
            "ERROR" => Some(LogLevel::ERROR),
            "DEBUG" => Some(LogLevel::DEBUG),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<LogLevel, usize>,
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
                self.add_entry(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let level = LogLevel::from_str(parts[1].trim())?;
        
        Some(LogEntry {
            timestamp: parts[0].trim().to_string(),
            level,
            source: parts[2].trim().to_string(),
            message: parts[3].trim().to_string(),
        })
    }

    fn add_entry(&mut self, entry: LogEntry) {
        *self.level_counts.entry(entry.level).or_insert(0) += 1;
        self.entries.push(entry);
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_summary(&self) -> HashMap<LogLevel, usize> {
        self.level_counts.clone()
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut analyzer = LogAnalyzer::new();
        let log_line = "2023-10-01 12:00:00 | INFO | server | User login successful";
        
        let entry = analyzer.parse_log_line(log_line).unwrap();
        assert_eq!(entry.level, LogLevel::INFO);
        assert_eq!(entry.source, "server");
        assert_eq!(entry.message, "User login successful");
    }

    #[test]
    fn test_filter_by_level() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 12:00:00 | INFO | server | User login").unwrap();
        writeln!(temp_file, "2023-10-01 12:01:00 | ERROR | db | Connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 12:02:00 | INFO | server | User logout").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        let errors = analyzer.filter_by_level(LogLevel::ERROR);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Connection failed");
    }

    #[test]
    fn test_summary() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 12:00:00 | INFO | server | Test").unwrap();
        writeln!(temp_file, "2023-10-01 12:01:00 | ERROR | db | Test").unwrap();
        writeln!(temp_file, "2023-10-01 12:02:00 | INFO | server | Test").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        let summary = analyzer.get_summary();
        assert_eq!(*summary.get(&LogLevel::INFO).unwrap(), 2);
        assert_eq!(*summary.get(&LogLevel::ERROR).unwrap(), 1);
    }
}