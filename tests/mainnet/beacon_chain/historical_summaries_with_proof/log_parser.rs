use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub struct LogParser {
    error_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        let pattern = r"ERROR\s+\[(.*?)\]\s+(.*)";
        LogParser {
            error_pattern: Regex::new(pattern).unwrap(),
        }
    }

    pub fn parse_file(&self, path: &str) -> io::Result<Vec<String>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = self.error_pattern.captures(&line) {
                let timestamp = captures.get(1).map_or("", |m| m.as_str());
                let message = captures.get(2).map_or("", |m| m.as_str());
                errors.push(format!("{}: {}", timestamp, message));
            }
        }

        Ok(errors)
    }

    pub fn count_errors(&self, path: &str) -> io::Result<usize> {
        let errors = self.parse_file(path)?;
        Ok(errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_error_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO [2023-10-05T10:30:00] Application started").unwrap();
        writeln!(temp_file, "ERROR [2023-10-05T10:31:00] Database connection failed").unwrap();
        writeln!(temp_file, "WARN [2023-10-05T10:32:00] High memory usage").unwrap();
        writeln!(temp_file, "ERROR [2023-10-05T10:33:00] File not found: config.yaml").unwrap();

        let parser = LogParser::new();
        let errors = parser.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Database connection failed"));
        assert!(errors[1].contains("File not found: config.yaml"));
    }

    #[test]
    fn test_count_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ERROR [2023-10-05T10:31:00] Test error 1").unwrap();
        writeln!(temp_file, "INFO [2023-10-05T10:32:00] Normal operation").unwrap();
        writeln!(temp_file, "ERROR [2023-10-05T10:33:00] Test error 2").unwrap();

        let parser = LogParser::new();
        let count = parser.count_errors(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(count, 2);
    }
}