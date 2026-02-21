use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

#[derive(Debug)]
struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    unique_messages: HashMap<String, usize>,
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_messages: HashMap::new(),
        }
    }

    fn add_entry(&mut self, entry: &LogEntry) {
        self.total_entries += 1;
        match entry.level.as_str() {
            "ERROR" => self.error_count += 1,
            "WARN" => self.warning_count += 1,
            "INFO" => self.info_count += 1,
            _ => {}
        }
        *self.unique_messages.entry(entry.message.clone()).or_insert(0) += 1;
    }

    fn display(&self) {
        println!("Log Analysis Summary:");
        println!("Total entries: {}", self.total_entries);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        println!("\nUnique messages (count):");
        for (message, count) in &self.unique_messages {
            println!("  {}: {}", message, count);
        }
    }
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let re = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)").ok()?;
    let caps = re.captures(line)?;
    
    Some(LogEntry {
        timestamp: caps[1].to_string(),
        level: caps[2].to_string(),
        message: caps[3].to_string(),
    })
}

fn analyze_log_file(file_path: &str) -> Result<LogSummary, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut summary = LogSummary::new();

    for line in reader.lines() {
        let line = line?;
        if let Some(entry) = parse_log_line(&line) {
            summary.add_entry(&entry);
        }
    }

    Ok(summary)
}

fn main() {
    let file_path = "application.log";
    match analyze_log_file(file_path) {
        Ok(summary) => summary.display(),
        Err(e) => eprintln!("Error analyzing log file: {}", e),
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_patterns: HashMap<String, usize>,
    total_lines: usize,
    time_range: (String, String),
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_patterns: HashMap::new(),
            total_lines: 0,
            time_range: (String::new(), String::new()),
        }
    }

    pub fn analyze_file(&mut self, filepath: &str) -> Result<(), String> {
        let file = File::open(filepath)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        let error_regex = Regex::new(r"ERROR|FATAL|CRITICAL").unwrap();
        let timestamp_regex = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();

        for line in reader.lines() {
            let line_content = line.map_err(|e| format!("Failed to read line: {}", e))?;
            self.total_lines += 1;

            if error_regex.is_match(&line_content) {
                let error_type = self.extract_error_type(&line_content);
                *self.error_patterns.entry(error_type).or_insert(0) += 1;
            }

            if let Some(timestamp) = timestamp_regex.find(&line_content) {
                let ts = timestamp.as_str().to_string();
                if self.time_range.0.is_empty() || ts < self.time_range.0 {
                    self.time_range.0 = ts;
                }
                if self.time_range.1.is_empty() || ts > self.time_range.1 {
                    self.time_range.1 = ts;
                }
            }
        }

        Ok(())
    }

    fn extract_error_type(&self, line: &str) -> String {
        let patterns = [
            ("DatabaseError", r"database|sql|connection"),
            ("NetworkError", r"network|timeout|connection refused"),
            ("AuthError", r"authentication|authorization|permission"),
            ("SystemError", r"memory|disk|resource"),
        ];

        for (name, pattern) in patterns.iter() {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(line) {
                return name.to_string();
            }
        }

        "UnknownError".to_string()
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total log lines analyzed: {}\n", self.total_lines));
        report.push_str(&format!("Time range: {} to {}\n", self.time_range.0, self.time_range.1));
        report.push_str("\nError distribution:\n");

        for (error_type, count) in &self.error_patterns {
            let percentage = (*count as f32 / self.total_lines as f32) * 100.0;
            report.push_str(&format!("  {}: {} ({:.2}%)\n", error_type, count, percentage));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut log_data = String::new();
        log_data.push_str("2024-01-15 10:30:00 INFO Application started\n");
        log_data.push_str("2024-01-15 10:31:00 ERROR Database connection failed\n");
        log_data.push_str("2024-01-15 10:32:00 WARNING High memory usage\n");
        log_data.push_str("2024-01-15 10:33:00 ERROR Network timeout occurred\n");
        log_data.push_str("2024-01-15 10:34:00 INFO Request processed\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();

        let report = analyzer.generate_report();
        assert!(report.contains("Total log lines analyzed: 5"));
        assert!(report.contains("DatabaseError"));
        assert!(report.contains("NetworkError"));
    }
}