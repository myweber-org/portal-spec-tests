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
}