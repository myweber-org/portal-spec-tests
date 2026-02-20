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
            self.analyze_line(&line, &mut stats);
        }
        
        Ok(stats)
    }

    fn analyze_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
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
        writeln!(temp_file, "INFO: User logged in").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        let report = analyzer.generate_report(&stats);
        
        assert_eq!(stats.get("info"), Some(&2));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("errors"), Some(&1));
        assert!(report.contains("Total log entries analyzed: 4"));
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                self.process_entry(entry);
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
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
        
        assert_eq!(analyzer.total_entries(), 3);
        
        let summary = analyzer.get_summary();
        assert_eq!(summary.get("INFO"), Some(&1));
        assert_eq!(summary.get("ERROR"), Some(&1));
        
        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Database connection failed");
        
        let search_results = analyzer.search_messages("memory");
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].level, "WARN");
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

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut stats = HashMap::new();
        stats.insert("total_lines".to_string(), 0);
        stats.insert("errors".to_string(), 0);
        stats.insert("warnings".to_string(), 0);
        stats.insert("info".to_string(), 0);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
            
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

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let total = stats.get("total_lines").unwrap_or(&0);
        let errors = stats.get("errors").unwrap_or(&0);
        let warnings = stats.get("warnings").unwrap_or(&0);
        let info = stats.get("info").unwrap_or(&0);
        
        format!(
            "Log Analysis Report:\n\
             Total Lines: {}\n\
             Errors: {}\n\
             Warnings: {}\n\
             Info Messages: {}",
            total, errors, warnings, info
        )
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
        writeln!(temp_file, "WARN: Low disk space").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: Processing complete").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(*stats.get("total_lines").unwrap(), 4);
        assert_eq!(*stats.get("errors").unwrap(), 1);
        assert_eq!(*stats.get("warnings").unwrap(), 1);
        assert_eq!(*stats.get("info").unwrap(), 2);
    }
}