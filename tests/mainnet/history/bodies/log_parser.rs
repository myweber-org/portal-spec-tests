
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
        let error_pattern = Regex::new(pattern).unwrap();
        LogParser { error_pattern }
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

    pub fn extract_timestamps(&self, log_lines: &[String]) -> Vec<String> {
        let timestamp_pattern = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();
        let mut timestamps = Vec::new();

        for line in log_lines {
            if let Some(captures) = timestamp_pattern.find(line) {
                timestamps.push(captures.as_str().to_string());
            }
        }

        timestamps
    }
}

pub fn analyze_error_frequency(errors: &[String]) -> std::collections::HashMap<String, usize> {
    let mut frequency = std::collections::HashMap::new();
    let error_type_pattern = Regex::new(r"ERROR: (\w+)").unwrap();

    for error in errors {
        if let Some(captures) = error_type_pattern.captures(error) {
            let error_type = captures.get(1).unwrap().as_str().to_string();
            *frequency.entry(error_type).or_insert(0) += 1;
        }
    }

    frequency
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_detection() {
        let parser = LogParser::new();
        let test_logs = vec![
            "2023-10-01 12:00:00 INFO: System started".to_string(),
            "2023-10-01 12:05:00 ERROR: Database connection failed".to_string(),
            "2023-10-01 12:10:00 WARNING: High memory usage".to_string(),
            "2023-10-01 12:15:00 FATAL: System crash detected".to_string(),
        ];

        let errors = parser.parse_file("test.log").unwrap_or(test_logs);
        let filtered_errors: Vec<String> = errors
            .iter()
            .filter(|line| parser.error_pattern.is_match(line))
            .cloned()
            .collect();

        assert_eq!(filtered_errors.len(), 2);
        assert!(filtered_errors[0].contains("ERROR"));
        assert!(filtered_errors[1].contains("FATAL"));
    }

    #[test]
    fn test_timestamp_extraction() {
        let parser = LogParser::new();
        let test_lines = vec![
            "2023-10-01 12:05:00 ERROR: Something went wrong".to_string(),
            "2023-10-01 12:10:00 INFO: Operation completed".to_string(),
        ];

        let timestamps = parser.extract_timestamps(&test_lines);
        assert_eq!(timestamps.len(), 2);
        assert_eq!(timestamps[0], "2023-10-01 12:05:00");
    }
}