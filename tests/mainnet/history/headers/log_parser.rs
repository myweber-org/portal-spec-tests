
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use regex::Regex;

pub struct LogParser {
    error_pattern: Regex,
    timestamp_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            error_pattern: Regex::new(r"ERROR|FATAL|CRITICAL").unwrap(),
            timestamp_pattern: Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap(),
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<String>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if self.error_pattern.is_match(&line) {
                errors.push(line);
            }
        }

        Ok(errors)
    }

    pub fn extract_timestamps(&self, log_entry: &str) -> Vec<String> {
        self.timestamp_pattern
            .find_iter(log_entry)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    pub fn analyze_errors(&self, errors: &[String]) -> usize {
        errors.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_detection() {
        let parser = LogParser::new();
        let test_log = "2023-10-05 12:30:45 ERROR Database connection failed";
        assert!(parser.error_pattern.is_match(test_log));
    }

    #[test]
    fn test_timestamp_extraction() {
        let parser = LogParser::new();
        let timestamps = parser.extract_timestamps("2023-10-05 12:30:45 ERROR 2023-10-05 12:31:00 FATAL");
        assert_eq!(timestamps.len(), 2);
    }
}