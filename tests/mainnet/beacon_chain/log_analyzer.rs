use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogSummary {
    total_lines: usize,
    error_count: usize,
    warning_count: usize,
    level_distribution: HashMap<String, usize>,
    unique_messages: Vec<String>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            level_distribution: HashMap::new(),
            unique_messages: Vec::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut summary = LogSummary::new();

        for line in reader.lines() {
            let line = line?;
            summary.process_line(&line);
        }

        Ok(summary)
    }

    fn process_line(&mut self, line: &str) {
        self.total_lines += 1;

        let line_lower = line.to_lowercase();
        
        if line_lower.contains("error") {
            self.error_count += 1;
            *self.level_distribution.entry("ERROR".to_string()).or_insert(0) += 1;
        } else if line_lower.contains("warning") {
            self.warning_count += 1;
            *self.level_distribution.entry("WARNING".to_string()).or_insert(0) += 1;
        } else if line_lower.contains("info") {
            *self.level_distribution.entry("INFO".to_string()).or_insert(0) += 1;
        } else if line_lower.contains("debug") {
            *self.level_distribution.entry("DEBUG".to_string()).or_insert(0) += 1;
        }

        if line.len() > 50 && !self.unique_messages.contains(&line.to_string()) {
            self.unique_messages.push(line.to_string());
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Level distribution: {:?}", self.level_distribution);
        println!("Unique messages found: {}", self.unique_messages.len());
    }
}

pub fn find_pattern_in_logs<P: AsRef<Path>>(path: P, pattern: &str) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut matches = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.contains(pattern) {
            matches.push(line);
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARNING: Low memory").unwrap();
        writeln!(temp_file, "ERROR: Connection failed").unwrap();
        writeln!(temp_file, "DEBUG: Processing data").unwrap();
        writeln!(temp_file, "INFO: Operation completed").unwrap();

        let summary = LogSummary::analyze_file(temp_file.path()).unwrap();
        assert_eq!(summary.total_lines, 5);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.level_distribution.get("INFO").unwrap(), &2);
    }

    #[test]
    fn test_pattern_search() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: Connected to database").unwrap();
        writeln!(temp_file, "ERROR: Timeout occurred").unwrap();

        let matches = find_pattern_in_logs(temp_file.path(), "ERROR").unwrap();
        assert_eq!(matches.len(), 2);
        assert!(matches[0].contains("ERROR"));
        assert!(matches[1].contains("ERROR"));
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
                self.process_entry(entry);
            }
        }

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

    fn process_entry(&mut self, entry: LogEntry) {
        *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        self.entries.push(entry);
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, usize> {
        self.level_counts.clone()
    }

    pub fn count_entries(&self) -> usize {
        self.entries.len()
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
        writeln!(temp_file, "2023-10-01T10:00:00 INFO Application started").unwrap();
        writeln!(temp_file, "2023-10-01T10:01:00 ERROR Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01T10:02:00 WARN High memory usage detected").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.count_entries(), 3);
        
        let stats = analyzer.get_statistics();
        assert_eq!(stats.get("INFO"), Some(&1));
        assert_eq!(stats.get("ERROR"), Some(&1));
        assert_eq!(stats.get("WARN"), Some(&1));
        
        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Database connection failed");
        
        let search_results = analyzer.search_messages("memory");
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].level, "WARN");
    }
}