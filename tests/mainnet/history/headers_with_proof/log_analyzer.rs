use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

pub struct LogAnalyzer {
    counts: HashMap<LogLevel, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            counts: HashMap::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line);
        }

        Ok(())
    }

    fn process_line(&mut self, line: &str) {
        let level = self.detect_log_level(line);
        *self.counts.entry(level).or_insert(0) += 1;
    }

    fn detect_log_level(&self, line: &str) -> LogLevel {
        let line_lower = line.to_lowercase();
        
        if line_lower.contains("error") || line_lower.contains("err") {
            LogLevel::Error
        } else if line_lower.contains("warning") || line_lower.contains("warn") {
            LogLevel::Warning
        } else if line_lower.contains("debug") {
            LogLevel::Debug
        } else {
            LogLevel::Info
        }
    }

    pub fn get_counts(&self) -> &HashMap<LogLevel, usize> {
        &self.counts
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("=====================");
        
        let total: usize = self.counts.values().sum();
        
        for (level, count) in &self.counts {
            let percentage = if total > 0 {
                (*count as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            println!("{:?}: {} ({:.1}%)", level, count, percentage);
        }
        
        println!("Total logs: {}", total);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_log_level() {
        let analyzer = LogAnalyzer::new();
        
        assert_eq!(analyzer.detect_log_level("ERROR: Something went wrong"), LogLevel::Error);
        assert_eq!(analyzer.detect_log_level("WARN: This might be an issue"), LogLevel::Warning);
        assert_eq!(analyzer.detect_log_level("DEBUG: Detailed information"), LogLevel::Debug);
        assert_eq!(analyzer.detect_log_level("INFO: Normal operation"), LogLevel::Info);
    }

    #[test]
    fn test_process_line() {
        let mut analyzer = LogAnalyzer::new();
        
        analyzer.process_line("ERROR: Failed to connect");
        analyzer.process_line("WARNING: High memory usage");
        analyzer.process_line("INFO: Server started");
        analyzer.process_line("ERROR: Database timeout");
        
        let counts = analyzer.get_counts();
        assert_eq!(counts.get(&LogLevel::Error), Some(&2));
        assert_eq!(counts.get(&LogLevel::Warning), Some(&1));
        assert_eq!(counts.get(&LogLevel::Info), Some(&1));
    }
}