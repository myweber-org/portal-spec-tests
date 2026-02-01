use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub unique_errors: HashMap<String, usize>,
    pub time_range: (String, String),
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_errors: HashMap::new(),
            time_range: (String::new(), String::new()),
        }
    }
}

pub fn analyze_log_file<P: AsRef<Path>>(path: P) -> Result<LogSummary, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut summary = LogSummary::new();
    let mut first_timestamp = String::new();
    let mut last_timestamp = String::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        summary.total_lines += 1;

        if line_num == 0 {
            first_timestamp = extract_timestamp(&line).unwrap_or_default();
        }
        last_timestamp = extract_timestamp(&line).unwrap_or_default();

        if line.contains("ERROR") {
            summary.error_count += 1;
            let error_key = extract_error_type(&line);
            *summary.unique_errors.entry(error_key).or_insert(0) += 1;
        } else if line.contains("WARN") {
            summary.warning_count += 1;
        } else if line.contains("INFO") {
            summary.info_count += 1;
        }
    }

    summary.time_range = (first_timestamp, last_timestamp);
    Ok(summary)
}

fn extract_timestamp(line: &str) -> Option<String> {
    line.split_whitespace()
        .next()
        .filter(|s| s.contains(':'))
        .map(|s| s.to_string())
}

fn extract_error_type(line: &str) -> String {
    line.split("ERROR")
        .nth(1)
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("unknown")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 10:00:00 INFO Application started").unwrap();
        writeln!(temp_file, "2023-10-01 10:01:00 ERROR Database: Connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 10:02:00 WARN Cache: Memory low").unwrap();
        writeln!(temp_file, "2023-10-01 10:03:00 ERROR Database: Timeout occurred").unwrap();

        let summary = analyze_log_file(temp_file.path()).unwrap();
        
        assert_eq!(summary.total_lines, 4);
        assert_eq!(summary.error_count, 2);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 1);
        assert_eq!(summary.unique_errors.get("Database"), Some(&2));
        assert_eq!(summary.time_range.0, "2023-10-01");
        assert_eq!(summary.time_range.1, "2023-10-01");
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warn_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warn_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut stats = HashMap::new();
        stats.insert("total_lines".to_string(), 0);
        stats.insert("errors".to_string(), 0);
        stats.insert("warnings".to_string(), 0);
        stats.insert("info".to_string(), 0);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| e.to_string())?;
            
            *stats.get_mut("total_lines").unwrap() += 1;
            
            if self.error_pattern.is_match(&line) {
                *stats.get_mut("errors").unwrap() += 1;
            } else if self.warn_pattern.is_match(&line) {
                *stats.get_mut("warnings").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.get_mut("info").unwrap() += 1;
            }
        }

        Ok(stats)
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let total = stats.get("total_lines").unwrap_or(&0);
        let errors = stats.get("errors").unwrap_or(&0);
        let warnings = stats.get("warnings").unwrap_or(&0);
        let info = stats.get("info").unwrap_or(&0);
        
        format!(
            "Log Analysis Report:\n\
            Total Lines: {}\n\
            Errors: {}\n\
            Warnings: {}\n\
            Info Messages: {}\n\
            Error Rate: {:.2}%",
            total,
            errors,
            warnings,
            info,
            (*errors as f64 / *total as f64) * 100.0
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
        writeln!(temp_file, "WARN: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: User login successful").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(*stats.get("total_lines").unwrap(), 4);
        assert_eq!(*stats.get("errors").unwrap(), 1);
        assert_eq!(*stats.get("warnings").unwrap(), 1);
        assert_eq!(*stats.get("info").unwrap(), 2);
    }
}