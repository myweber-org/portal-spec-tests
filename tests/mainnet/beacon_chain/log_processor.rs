use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub struct LogProcessor {
    error_pattern: Regex,
}

impl LogProcessor {
    pub fn new() -> Self {
        let pattern = r"ERROR|FATAL|CRITICAL|FAILED";
        let regex = Regex::new(pattern).expect("Invalid regex pattern");
        LogProcessor {
            error_pattern: regex,
        }
    }

    pub fn process_log_file(&self, file_path: &str) -> io::Result<Vec<String>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line_content = line?;
            if self.error_pattern.is_match(&line_content) {
                errors.push(format!("Line {}: {}", line_number + 1, line_content));
            }
        }

        Ok(errors)
    }

    pub fn count_errors(&self, file_path: &str) -> io::Result<usize> {
        let errors = self.process_log_file(file_path)?;
        Ok(errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_error_extraction() {
        let processor = LogProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: System started").unwrap();
        writeln!(temp_file, "ERROR: Disk full").unwrap();
        writeln!(temp_file, "WARNING: High memory usage").unwrap();
        writeln!(temp_file, "FATAL: Kernel panic").unwrap();

        let errors = processor.process_log_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("ERROR: Disk full"));
        assert!(errors[1].contains("FATAL: Kernel panic"));
    }

    #[test]
    fn test_error_count() {
        let processor = LogProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Test").unwrap();
        writeln!(temp_file, "ERROR: Test error").unwrap();
        writeln!(temp_file, "CRITICAL: Something bad").unwrap();

        let count = processor.count_errors(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 2);
    }
}