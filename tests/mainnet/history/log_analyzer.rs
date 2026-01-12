use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR: (.+)").unwrap(),
        }
    }

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut error_counts = HashMap::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if let Some(captures) = self.error_pattern.captures(&line) {
                if let Some(error_msg) = captures.get(1) {
                    *error_counts.entry(error_msg.as_str().to_string()).or_insert(0) += 1;
                }
            }
        }

        Ok(error_counts)
    }

    pub fn print_summary(&self, counts: &HashMap<String, usize>) {
        if counts.is_empty() {
            println!("No errors found in log file.");
            return;
        }

        println!("Error Summary:");
        println!("{:<50} | {:>10}", "Error Message", "Count");
        println!("{:-<50}-+-{:-<10}", "", "");

        let mut sorted_errors: Vec<_> = counts.iter().collect();
        sorted_errors.sort_by(|a, b| b.1.cmp(a.1));

        for (error, count) in sorted_errors {
            println!("{:<50} | {:>10}", error, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_analyze_log_file() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "ERROR: Invalid user input").unwrap();
        
        let result = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(result.get("Database connection failed"), Some(&2));
        assert_eq!(result.get("Invalid user input"), Some(&1));
        assert_eq!(result.len(), 2);
    }
}