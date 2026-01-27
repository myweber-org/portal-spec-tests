
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use regex::Regex;

pub struct LogParser {
    error_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        let pattern = r"ERROR|FATAL|CRITICAL|FAILED";
        let error_pattern = Regex::new(pattern).expect("Invalid regex pattern");
        
        LogParser { error_pattern }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<String>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if self.error_pattern.is_match(&line) {
                errors.push(format!("Line {}: {}", line_num + 1, line));
            }
        }

        Ok(errors)
    }

    pub fn find_errors(&self, log_content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        for (line_num, line) in log_content.lines().enumerate() {
            if self.error_pattern.is_match(line) {
                errors.push(format!("Line {}: {}", line_num + 1, line));
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_detection() {
        let parser = LogParser::new();
        let log_data = "INFO: Application started\nERROR: Database connection failed\nWARN: High memory usage\nFATAL: System crash";
        
        let errors = parser.find_errors(log_data);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("ERROR: Database connection failed"));
        assert!(errors[1].contains("FATAL: System crash"));
    }

    #[test]
    fn test_no_errors() {
        let parser = LogParser::new();
        let log_data = "INFO: Application started\nDEBUG: Processing request\nWARN: Cache miss";
        
        let errors = parser.find_errors(log_data);
        assert!(errors.is_empty());
    }
}