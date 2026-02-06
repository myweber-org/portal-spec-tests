use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_patterns: HashMap<String, usize>,
    warning_patterns: HashMap<String, usize>,
    info_patterns: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_patterns: HashMap::new(),
            warning_patterns: HashMap::new(),
            info_patterns: HashMap::new(),
        }
    }

    pub fn analyze_file(&mut self, file_path: &str) -> Result<(), std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let error_re = Regex::new(r"ERROR: (.+)").unwrap();
        let warning_re = Regex::new(r"WARNING: (.+)").unwrap();
        let info_re = Regex::new(r"INFO: (.+)").unwrap();

        for line in reader.lines() {
            let line = line?;
            
            if let Some(caps) = error_re.captures(&line) {
                let msg = caps.get(1).unwrap().as_str().to_string();
                *self.error_patterns.entry(msg).or_insert(0) += 1;
            } else if let Some(caps) = warning_re.captures(&line) {
                let msg = caps.get(1).unwrap().as_str().to_string();
                *self.warning_patterns.entry(msg).or_insert(0) += 1;
            } else if let Some(caps) = info_re.captures(&line) {
                let msg = caps.get(1).unwrap().as_str().to_string();
                *self.info_patterns.entry(msg).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== LOG ANALYSIS REPORT ===\n\n");
        
        report.push_str("ERRORS:\n");
        for (msg, count) in &self.error_patterns {
            report.push_str(&format!("  {}: {} occurrences\n", msg, count));
        }
        
        report.push_str("\nWARNINGS:\n");
        for (msg, count) in &self.warning_patterns {
            report.push_str(&format!("  {}: {} occurrences\n", msg, count));
        }
        
        report.push_str("\nINFO MESSAGES:\n");
        for (msg, count) in &self.info_patterns {
            report.push_str(&format!("  {}: {} occurrences\n", msg, count));
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
        let mut log_data = NamedTempFile::new().unwrap();
        writeln!(log_data, "INFO: Application started").unwrap();
        writeln!(log_data, "WARNING: Memory usage high").unwrap();
        writeln!(log_data, "ERROR: Database connection failed").unwrap();
        writeln!(log_data, "INFO: User login successful").unwrap();
        writeln!(log_data, "ERROR: Database connection failed").unwrap();
        
        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(log_data.path().to_str().unwrap()).unwrap();
        
        let report = analyzer.generate_report();
        assert!(report.contains("Database connection failed: 2 occurrences"));
        assert!(report.contains("Memory usage high: 1 occurrences"));
        assert!(report.contains("User login successful: 1 occurrences"));
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

    pub fn parse_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures[1].to_string(),
                    level: captures[2].to_string(),
                    message: captures[3].to_string(),
                };
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    pub fn analyze_levels(&mut self) {
        self.level_counts.clear();
        for entry in &self.entries {
            *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
    }

    pub fn get_level_summary(&self) -> Vec<(String, usize)> {
        let mut summary: Vec<_> = self.level_counts.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        summary.sort_by(|a, b| b.1.cmp(&a.1));
        summary
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert_eq!(analyzer.total_entries(), 0);
    }

    #[test]
    fn test_level_filtering() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: "2024-01-01 10:00:00".to_string(),
            level: "ERROR".to_string(),
            message: "Test error".to_string(),
        });
        analyzer.entries.push(LogEntry {
            timestamp: "2024-01-01 10:01:00".to_string(),
            level: "INFO".to_string(),
            message: "Test info".to_string(),
        });

        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Test error");
    }
}