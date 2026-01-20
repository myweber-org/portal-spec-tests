use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    module: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    module_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            module_counts: HashMap::new(),
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
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 4 {
            return None;
        }

        Some(LogEntry {
            timestamp: parts[0].to_string(),
            level: parts[1].to_string(),
            module: parts[2].to_string(),
            message: parts[3].to_string(),
        })
    }

    fn add_entry(&mut self, entry: LogEntry) {
        *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        *self.module_counts.entry(entry.module.clone()).or_insert(0) += 1;
        self.entries.push(entry);
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_module(&self, module: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.module == module)
            .collect()
    }

    pub fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let level_summary: Vec<String> = self
            .level_counts
            .iter()
            .map(|(level, count)| format!("{}: {}", level, count))
            .collect();
        let module_summary: Vec<String> = self
            .module_counts
            .iter()
            .map(|(module, count)| format!("{}: {}", module, count))
            .collect();

        format!(
            "Total entries: {}\nLevels: {}\nModules: {}",
            total_entries,
            level_summary.join(", "),
            module_summary.join(", ")
        )
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
    fn test_log_analysis() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01T10:00:00 INFO network Connected to server").unwrap();
        writeln!(temp_file, "2023-10-01T10:01:00 ERROR database Connection failed").unwrap();
        writeln!(temp_file, "2023-10-01T10:02:00 WARN network Timeout detected").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.filter_by_module("network").len(), 2);
        
        let summary = analyzer.get_summary();
        assert!(summary.contains("Total entries: 3"));
        assert!(summary.contains("INFO: 1"));
        assert!(summary.contains("network: 2"));
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

    pub fn analyze_log_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();
        
        for line in reader.lines() {
            let line_content = line.map_err(|e| format!("Failed to read line: {}", e))?;
            self.analyze_line(&line_content, &mut stats);
        }
        
        Ok(stats)
    }
    
    fn analyze_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("errors".to_string()).or_insert(0) += 1;
        } else if self.warning_pattern.is_match(line) {
            *stats.entry("warnings".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("info_messages".to_string()).or_insert(0) += 1;
        }
    }
    
    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::from("Log Analysis Report\n");
        report.push_str("===================\n");
        
        for (category, count) in stats {
            report.push_str(&format!("{}: {}\n", category, count));
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
        writeln!(temp_file, "2023-10-01 INFO: Application started").unwrap();
        writeln!(temp_file, "2023-10-01 ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 WARN: High memory usage detected").unwrap();
        writeln!(temp_file, "2023-10-01 INFO: User login successful").unwrap();
        
        let stats = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("errors"), Some(&1));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("info_messages"), Some(&2));
    }
}