use std::collections::HashMap;
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
        let level = entry.level.clone();
        *self.level_counts.entry(level).or_insert(0) += 1;
        self.entries.push(entry);
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

    pub fn find_messages(&self, keyword: &str) -> Vec<&LogEntry> {
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
        writeln!(temp_file, "2023-10-01T10:00:00 INFO System started").unwrap();
        writeln!(temp_file, "2023-10-01T10:01:00 ERROR Connection failed").unwrap();
        writeln!(temp_file, "2023-10-01T10:02:00 WARN High memory usage").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.count_entries(), 3);
        
        let summary = analyzer.get_summary();
        assert_eq!(summary.get("INFO"), Some(&1));
        assert_eq!(summary.get("ERROR"), Some(&1));
        
        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Connection failed");
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_patterns: HashMap<String, Regex>,
    warning_patterns: HashMap<String, Regex>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        let mut error_patterns = HashMap::new();
        let mut warning_patterns = HashMap::new();

        error_patterns.insert(
            "connection_error".to_string(),
            Regex::new(r"connection.*failed|timeout|refused").unwrap(),
        );
        error_patterns.insert(
            "authentication_error".to_string(),
            Regex::new(r"auth.*failed|invalid.*credentials").unwrap(),
        );

        warning_patterns.insert(
            "deprecation_warning".to_string(),
            Regex::new(r"deprecated|will.*remove").unwrap(),
        );
        warning_patterns.insert(
            "resource_warning".to_string(),
            Regex::new(r"low.*memory|high.*cpu").unwrap(),
        );

        LogAnalyzer {
            error_patterns,
            warning_patterns,
        }
    }

    pub fn analyze_log_file(&self, file_path: &str) -> Result<LogSummary, String> {
        let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);

        let mut summary = LogSummary::new();
        let mut line_count = 0;

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
            line_count += 1;

            self.analyze_line(&line, &mut summary);
        }

        summary.total_lines = line_count;
        Ok(summary)
    }

    fn analyze_line(&self, line: &str, summary: &mut LogSummary) {
        for (error_type, pattern) in &self.error_patterns {
            if pattern.is_match(line) {
                summary.error_counts.entry(error_type.clone()).and_modify(|e| *e += 1).or_insert(1);
                summary.total_errors += 1;
            }
        }

        for (warning_type, pattern) in &self.warning_patterns {
            if pattern.is_match(line) {
                summary.warning_counts.entry(warning_type.clone()).and_modify(|w| *w += 1).or_insert(1);
                summary.total_warnings += 1;
            }
        }

        if line.contains("ERROR") {
            summary.error_lines.push(line.to_string());
        } else if line.contains("WARN") {
            summary.warning_lines.push(line.to_string());
        }
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub error_counts: HashMap<String, usize>,
    pub warning_counts: HashMap<String, usize>,
    pub error_lines: Vec<String>,
    pub warning_lines: Vec<String>,
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_lines: 0,
            total_errors: 0,
            total_warnings: 0,
            error_counts: HashMap::new(),
            warning_counts: HashMap::new(),
            error_lines: Vec::new(),
            warning_lines: Vec::new(),
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines processed: {}", self.total_lines);
        println!("Total errors: {}", self.total_errors);
        println!("Total warnings: {}", self.total_warnings);

        if !self.error_counts.is_empty() {
            println!("\nError breakdown:");
            for (error_type, count) in &self.error_counts {
                println!("  {}: {}", error_type, count);
            }
        }

        if !self.warning_counts.is_empty() {
            println!("\nWarning breakdown:");
            for (warning_type, count) in &self.warning_counts {
                println!("  {}: {}", warning_type, count);
            }
        }

        if !self.error_lines.is_empty() {
            println!("\nSample error lines (first 3):");
            for line in self.error_lines.iter().take(3) {
                println!("  {}", line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let analyzer = LogAnalyzer::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ERROR: connection failed to database").unwrap();
        writeln!(temp_file, "WARN: deprecated API will be removed").unwrap();
        writeln!(temp_file, "INFO: application started").unwrap();
        writeln!(temp_file, "ERROR: authentication failed for user").unwrap();

        let summary = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(summary.total_lines, 4);
        assert_eq!(summary.total_errors, 2);
        assert_eq!(summary.total_warnings, 1);
        assert_eq!(summary.error_counts.get("connection_error"), Some(&1));
        assert_eq!(summary.error_counts.get("authentication_error"), Some(&1));
        assert_eq!(summary.warning_counts.get("deprecation_warning"), Some(&1));
    }
}