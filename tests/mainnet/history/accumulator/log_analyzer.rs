use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warning_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warning_pattern: Regex::new(r"WARNING").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_log_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut summary = HashMap::new();
        
        summary.insert("total_lines".to_string(), 0);
        summary.insert("errors".to_string(), 0);
        summary.insert("warnings".to_string(), 0);
        summary.insert("info".to_string(), 0);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
            
            *summary.get_mut("total_lines").unwrap() += 1;
            
            if self.error_pattern.is_match(&line) {
                *summary.get_mut("errors").unwrap() += 1;
            } else if self.warning_pattern.is_match(&line) {
                *summary.get_mut("warnings").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *summary.get_mut("info").unwrap() += 1;
            }
        }
        
        Ok(summary)
    }

    pub fn generate_report(&self, summary: &HashMap<String, usize>) -> String {
        format!(
            "Log Analysis Report:\n\
             Total Lines: {}\n\
             Errors: {}\n\
             Warnings: {}\n\
             Info Messages: {}",
            summary.get("total_lines").unwrap_or(&0),
            summary.get("errors").unwrap_or(&0),
            summary.get("warnings").unwrap_or(&0),
            summary.get("info").unwrap_or(&0)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARNING: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: User logged in").unwrap();
        
        let summary = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(*summary.get("total_lines").unwrap(), 4);
        assert_eq!(*summary.get("errors").unwrap(), 1);
        assert_eq!(*summary.get("warnings").unwrap(), 1);
        assert_eq!(*summary.get("info").unwrap(), 2);
    }
}