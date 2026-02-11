use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
struct LogSummary {
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    unique_errors: HashMap<String, usize>,
    time_range: (String, String),
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_errors: HashMap::new(),
            time_range: (String::new(), String::new()),
        }
    }
}

fn analyze_log_file(file_path: &str) -> Result<LogSummary, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut summary = LogSummary::new();
    
    let error_regex = Regex::new(r"ERROR")?;
    let warning_regex = Regex::new(r"WARN")?;
    let info_regex = Regex::new(r"INFO")?;
    let timestamp_regex = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}")?;
    
    let mut first_timestamp = String::new();
    let mut last_timestamp = String::new();
    
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        
        if error_regex.is_match(&line) {
            summary.error_count += 1;
            let error_key = extract_error_pattern(&line);
            *summary.unique_errors.entry(error_key).or_insert(0) += 1;
        } else if warning_regex.is_match(&line) {
            summary.warning_count += 1;
        } else if info_regex.is_match(&line) {
            summary.info_count += 1;
        }
        
        if let Some(captures) = timestamp_regex.find(&line) {
            let timestamp = captures.as_str().to_string();
            if line_num == 0 {
                first_timestamp = timestamp.clone();
            }
            last_timestamp = timestamp;
        }
    }
    
    summary.time_range = (first_timestamp, last_timestamp);
    Ok(summary)
}

fn extract_error_pattern(line: &str) -> String {
    let error_pattern_regex = Regex::new(r"ERROR.*?:\s*(.*?)(?:\s+at\s|$)").unwrap();
    
    if let Some(captures) = error_pattern_regex.captures(line) {
        if let Some(error_msg) = captures.get(1) {
            return error_msg.as_str().to_string();
        }
    }
    
    "Unknown error pattern".to_string()
}

fn generate_report(summary: &LogSummary) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("Log Analysis Report\n"));
    report.push_str(&format!("===================\n"));
    report.push_str(&format!("Time Range: {} - {}\n", summary.time_range.0, summary.time_range.1));
    report.push_str(&format!("Total INFO entries: {}\n", summary.info_count));
    report.push_str(&format!("Total WARN entries: {}\n", summary.warning_count));
    report.push_str(&format!("Total ERROR entries: {}\n", summary.error_count));
    
    if !summary.unique_errors.is_empty() {
        report.push_str("\nUnique Error Patterns:\n");
        for (error, count) in &summary.unique_errors {
            report.push_str(&format!("  {} (occurrences: {})\n", error, count));
        }
    }
    
    report
}

fn main() {
    let file_path = "application.log";
    
    match analyze_log_file(file_path) {
        Ok(summary) => {
            let report = generate_report(&summary);
            println!("{}", report);
            
            if summary.error_count > 0 {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to analyze log file: {}", e);
            std::process::exit(1);
        }
    }
}use std::collections::HashMap;
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
            warning_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            self.analyze_line(&line, &mut stats);
        }

        Ok(stats)
    }

    fn analyze_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("errors".to_string()).or_insert(0) += 1;
        } else if self.warning_pattern.is_match(line) {
            *stats.entry("warnings".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("info".to_string()).or_insert(0) += 1;
        }
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::new();
        report.push_str("Log Analysis Report\n");
        report.push_str("===================\n");

        for (category, count) in stats {
            report.push_str(&format!("{}: {}\n", category, count));
        }

        let total: usize = stats.values().sum();
        report.push_str(&format!("\nTotal log entries analyzed: {}", total));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_counts() {
        let analyzer = LogAnalyzer::new();
        let mut stats = HashMap::new();

        analyzer.analyze_line("2024-01-15 ERROR: Database connection failed", &mut stats);
        analyzer.analyze_line("2024-01-15 WARN: High memory usage detected", &mut stats);
        analyzer.analyze_line("2024-01-15 INFO: Server started successfully", &mut stats);
        analyzer.analyze_line("2024-01-15 ERROR: File not found", &mut stats);

        assert_eq!(stats.get("errors"), Some(&2));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("info"), Some(&1));
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures[1].to_string(),
                    level: captures[2].to_string(),
                    message: captures[3].to_string(),
                };

                *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn get_level_summary(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:30:45 [INFO] Application started"
        ).unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:31:22 [ERROR] Failed to connect to database"
        ).unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:32:10 [WARN] High memory usage detected"
        ).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.total_entries(), 3);
        assert_eq!(analyzer.get_level_summary().get("INFO"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("ERROR"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("WARN"), Some(&1));
    }
}