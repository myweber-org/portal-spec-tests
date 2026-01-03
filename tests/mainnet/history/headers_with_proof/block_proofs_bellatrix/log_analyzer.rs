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

    pub fn analyze_log_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            
            if self.error_pattern.is_match(&line) {
                *stats.entry("ERROR".to_string()).or_insert(0) += 1;
            } else if self.warn_pattern.is_match(&line) {
                *stats.entry("WARN".to_string()).or_insert(0) += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.entry("INFO".to_string()).or_insert(0) += 1;
            }
        }
        
        Ok(stats)
    }
    
    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::from("Log Analysis Report\n");
        report.push_str("===================\n");
        
        let total: usize = stats.values().sum();
        report.push_str(&format!("Total log entries: {}\n", total));
        
        for (level, count) in stats {
            let percentage = if total > 0 {
                (*count as f64 / total as f64) * 100.0
            } else {
                0.0
            };
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
        writeln!(temp_file, "2024-01-15 INFO: Application started").unwrap();
        writeln!(temp_file, "2024-01-15 WARN: Disk space low").unwrap();
        writeln!(temp_file, "2024-01-15 ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "2024-01-15 INFO: User login successful").unwrap();
        
        let stats = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("INFO"), Some(&2));
        assert_eq!(stats.get("WARN"), Some(&1));
        assert_eq!(stats.get("ERROR"), Some(&1));
        
        let report = analyzer.generate_report(&stats);
        assert!(report.contains("Total log entries: 4"));
        assert!(report.contains("INFO: 2"));
    }
}