use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub fn extract_errors(log_path: &str) -> io::Result<Vec<String>> {
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let error_pattern = Regex::new(r"(?i)error|exception|fail|critical|alert").unwrap();
    let mut errors = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if error_pattern.is_match(&line) {
            errors.push(format!("Line {}: {}", line_num + 1, line));
        }
    }

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage").unwrap();
        writeln!(temp_file, "CRITICAL: System outage detected").unwrap();

        let errors = extract_errors(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("ERROR: Database connection failed"));
        assert!(errors[1].contains("CRITICAL: System outage detected"));
    }
}