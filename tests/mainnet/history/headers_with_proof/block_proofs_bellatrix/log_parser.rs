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

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if self.error_pattern.is_match(&line) {
                errors.push(format!("Line {}: {}", line_num + 1, line));
            }
        }

        Ok(errors)
    }

    pub fn count_errors<P: AsRef<Path>>(&self, path: P) -> io::Result<usize> {
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
    fn test_error_detection() {
        let parser = LogParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage").unwrap();
        writeln!(temp_file, "FATAL: System crash detected").unwrap();
        
        let errors = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("ERROR"));
        assert!(errors[1].contains("FATAL"));
    }

    #[test]
    fn test_no_errors() {
        let parser = LogParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "INFO: System operational").unwrap();
        writeln!(temp_file, "DEBUG: Processing request").unwrap();
        
        let errors = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(errors.len(), 0);
    }
}