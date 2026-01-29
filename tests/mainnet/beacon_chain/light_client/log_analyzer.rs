use std::collections::HashMap;
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
            error_pattern: Regex::new(r"(?i)error").unwrap(),
            warn_pattern: Regex::new(r"(?i)warn").unwrap(),
            info_pattern: Regex::new(r"(?i)info").unwrap(),
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
            Total lines: {}\n\
            Errors: {}\n\
            Warnings: {}\n\
            Info messages: {}\n\
            Error rate: {:.2}%",
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
    fn test_analyze_logs() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Low memory").unwrap();
        writeln!(temp_file, "ERROR: Failed to connect").unwrap();
        writeln!(temp_file, "INFO: Processing data").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(*stats.get("total_lines").unwrap(), 4);
        assert_eq!(*stats.get("errors").unwrap(), 1);
        assert_eq!(*stats.get("warnings").unwrap(), 1);
        assert_eq!(*stats.get("info").unwrap(), 2);
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

    pub fn analyze_log_file(&self, file_path: &str) -> Result<LogSummary, String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary::new();
        
        for line_result in reader.lines() {
            let line = line_result.map_err(|e| e.to_string())?;
            self.process_line(&line, &mut summary);
        }
        
        Ok(summary)
    }

    fn process_line(&self, line: &str, summary: &mut LogSummary) {
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.errors.push(line.to_string());
        } else if self.warning_pattern.is_match(line) {
            summary.warning_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
        
        summary.total_lines += 1;
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub errors: Vec<String>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            errors: Vec::new(),
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        
        if !self.errors.is_empty() {
            println!("\nRecent errors:");
            for error in self.errors.iter().take(5) {
                println!("  - {}", error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: Processing complete").unwrap();
        
        let analyzer = LogAnalyzer::new();
        let summary = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(summary.total_lines, 4);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 2);
        assert_eq!(summary.errors.len(), 1);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    component: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    error_count: usize,
    warning_count: usize,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(?P<timestamp>[\d\-:T.]+Z)\] (?P<level>\w+) \[(?P<component>[\w:]+)\] (?P<message>.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp_str = captures.name("timestamp").unwrap().as_str();
                let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?.with_timezone(&Utc);
                let level = captures.name("level").unwrap().as_str().to_string();
                let component = captures.name("component").unwrap().as_str().to_string();
                let message = captures.name("message").unwrap().as_str().to_string();

                match level.as_str() {
                    "ERROR" => self.error_count += 1,
                    "WARNING" => self.warning_count += 1,
                    _ => {}
                }

                self.entries.push(LogEntry {
                    timestamp,
                    level,
                    component,
                    message,
                });
            }
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_component_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for entry in &self.entries {
            *stats.entry(entry.component.clone()).or_insert(0) += 1;
        }
        stats
    }

    pub fn get_error_count(&self) -> usize {
        self.error_count
    }

    pub fn get_warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn get_total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn find_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert_eq!(analyzer.get_total_entries(), 0);
        assert_eq!(analyzer.get_error_count(), 0);
    }
}use std::collections::HashMap;
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

    for (index, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        summary.total_lines += 1;

        if let Some(timestamp) = extract_timestamp(&line) {
            if index == 0 {
                first_timestamp = timestamp.clone();
            }
            last_timestamp = timestamp;
        }

        classify_log_line(&line, &mut summary);
    }

    summary.time_range = (first_timestamp, last_timestamp);
    Ok(summary)
}

fn extract_timestamp(line: &str) -> Option<String> {
    if line.len() > 20 {
        let potential_timestamp = &line[0..19];
        if potential_timestamp.contains('-') && potential_timestamp.contains(':') {
            return Some(potential_timestamp.to_string());
        }
    }
    None
}

fn classify_log_line(line: &str, summary: &mut LogSummary) {
    let line_lower = line.to_lowercase();
    
    if line_lower.contains("error") {
        summary.error_count += 1;
        if let Some(error_msg) = extract_error_message(line) {
            *summary.unique_errors.entry(error_msg).or_insert(0) += 1;
        }
    } else if line_lower.contains("warning") {
        summary.warning_count += 1;
    } else if line_lower.contains("info") {
        summary.info_count += 1;
    }
}

fn extract_error_message(line: &str) -> Option<String> {
    let error_keywords = ["error:", "exception:", "failed:", "unable to"];
    
    for keyword in error_keywords.iter() {
        if let Some(pos) = line.to_lowercase().find(keyword) {
            let start = pos + keyword.len();
            let end = line.len().min(start + 100);
            return Some(line[start..end].trim().to_string());
        }
    }
    None
}

pub fn print_summary(summary: &LogSummary) {
    println!("Log Analysis Summary");
    println!("====================");
    println!("Total lines: {}", summary.total_lines);
    println!("Errors: {}", summary.error_count);
    println!("Warnings: {}", summary.warning_count);
    println!("Info messages: {}", summary.info_count);
    println!("Time range: {} - {}", summary.time_range.0, summary.time_range.1);
    
    if !summary.unique_errors.is_empty() {
        println!("\nUnique Errors:");
        for (error, count) in &summary.unique_errors {
            println!("  {} (occurrences: {})", error, count);
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
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_content = "2023-10-01 10:00:00 INFO Application started\n\
                          2023-10-01 10:01:00 ERROR: Database connection failed\n\
                          2023-10-01 10:02:00 WARNING: High memory usage\n\
                          2023-10-01 10:03:00 INFO: Processing complete\n";
        
        writeln!(temp_file, "{}", log_content).unwrap();
        
        let summary = analyze_log_file(temp_file.path()).unwrap();
        
        assert_eq!(summary.total_lines, 4);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 2);
        assert_eq!(summary.time_range.0, "2023-10-01 10:00:00");
        assert_eq!(summary.time_range.1, "2023-10-01 10:03:00");
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

    pub fn analyze_file(&self, path: &str) -> Result<LogSummary, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            self.process_line(&line, &mut summary);
        }
        
        Ok(summary)
    }
    
    fn process_line(&self, line: &str, summary: &mut LogSummary) {
        summary.total_lines += 1;
        
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.error_lines.push(line.to_string());
        } else if self.warn_pattern.is_match(line) {
            summary.warn_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub error_lines: Vec<String>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            error_lines: Vec::new(),
        }
    }
    
    pub fn print_report(&self) {
        println!("Log Analysis Report");
        println!("===================");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warn_count);
        println!("Info messages: {}", self.info_count);
        
        if !self.error_lines.is_empty() {
            println!("\nError lines found:");
            for (i, line) in self.error_lines.iter().enumerate().take(5) {
                println!("  {}. {}", i + 1, line);
            }
            if self.error_lines.len() > 5 {
                println!("  ... and {} more errors", self.error_lines.len() - 5);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Low memory").unwrap();
        writeln!(temp_file, "ERROR: Failed to connect").unwrap();
        writeln!(temp_file, "INFO: Processing data").unwrap();
        writeln!(temp_file, "ERROR: Timeout occurred").unwrap();
        
        let analyzer = LogAnalyzer::new();
        let summary = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(summary.total_lines, 5);
        assert_eq!(summary.error_count, 2);
        assert_eq!(summary.warn_count, 1);
        assert_eq!(summary.info_count, 2);
        assert_eq!(summary.error_lines.len(), 2);
    }
}