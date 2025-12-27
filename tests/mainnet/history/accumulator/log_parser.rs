use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub struct LogParser {
    error_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        let pattern = r"ERROR\s+\[.*?\]\s+(.+)";
        LogParser {
            error_pattern: Regex::new(pattern).unwrap(),
        }
    }

    pub fn parse_file(&self, file_path: &str) -> io::Result<Vec<String>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = self.error_pattern.captures(&line) {
                if let Some(error_msg) = captures.get(1) {
                    errors.push(error_msg.as_str().to_string());
                }
            }
        }

        Ok(errors)
    }

    pub fn count_errors(&self, file_path: &str) -> io::Result<usize> {
        let errors = self.parse_file(file_path)?;
        Ok(errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_error_messages() {
        let parser = LogParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO [module_a] System started").unwrap();
        writeln!(temp_file, "ERROR [module_b] Database connection failed").unwrap();
        writeln!(temp_file, "WARN [module_c] High memory usage").unwrap();
        writeln!(temp_file, "ERROR [module_a] Invalid user input detected").unwrap();

        let errors = parser.parse_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors.contains(&"Database connection failed".to_string()));
        assert!(errors.contains(&"Invalid user input detected".to_string()));
    }

    #[test]
    fn test_error_count() {
        let parser = LogParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        for i in 0..5 {
            writeln!(temp_file, "ERROR [test] Error number {}", i).unwrap();
        }

        let count = parser.count_errors(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 5);
    }
}