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

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    fn parse_log_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp = captures[1].to_string();
                let level = captures[2].to_string();
                let message = captures[3].to_string();

                let entry = LogEntry {
                    timestamp,
                    level: level.clone(),
                    message,
                };

                self.entries.push(entry);
                *self.level_counts.entry(level).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    fn generate_summary(&self) {
        println!("Log Analysis Summary");
        println!("====================");
        println!("Total entries: {}", self.entries.len());
        println!("\nLog level distribution:");

        for (level, count) in &self.level_counts {
            let percentage = (*count as f64 / self.entries.len() as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", level, count, percentage);
        }

        if let Some(error_count) = self.level_counts.get("ERROR") {
            if *error_count > 0 {
                println!("\nError entries found:");
                for entry in &self.entries {
                    if entry.level == "ERROR" {
                        println!("  [{}] {}", entry.timestamp, entry.message);
                    }
                }
            }
        }
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
}

fn main() {
    let mut analyzer = LogAnalyzer::new();

    match analyzer.parse_log_file("application.log") {
        Ok(_) => {
            analyzer.generate_summary();
            
            let warnings = analyzer.filter_by_level("WARN");
            if !warnings.is_empty() {
                println!("\nWarning entries ({} total):", warnings.len());
                for entry in warnings.iter().take(5) {
                    println!("  [{}] {}", entry.timestamp, entry.message);
                }
            }
        }
        Err(e) => eprintln!("Failed to parse log file: {}", e),
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LogSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

pub struct LogAnalyzer {
    log_counts: HashMap<LogSeverity, usize>,
    total_lines: usize,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            log_counts: HashMap::new(),
            total_lines: 0,
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.total_lines += 1;
            
            let severity = self.detect_severity(&line);
            *self.log_counts.entry(severity).or_insert(0) += 1;
        }

        Ok(())
    }

    fn detect_severity(&self, line: &str) -> LogSeverity {
        let line_lower = line.to_lowercase();
        
        if line_lower.contains("critical") || line_lower.contains("fatal") {
            LogSeverity::Critical
        } else if line_lower.contains("error") || line_lower.contains("err") {
            LogSeverity::Error
        } else if line_lower.contains("warning") || line_lower.contains("warn") {
            LogSeverity::Warning
        } else if line_lower.contains("debug") {
            LogSeverity::Debug
        } else {
            LogSeverity::Info
        }
    }

    pub fn get_statistics(&self) -> HashMap<LogSeverity, usize> {
        self.log_counts.clone()
    }

    pub fn get_total_lines(&self) -> usize {
        self.total_lines
    }

    pub fn filter_by_severity<P: AsRef<Path>>(
        path: P,
        severity: LogSeverity,
    ) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut filtered_lines = Vec::new();
        let analyzer = LogAnalyzer::new();

        for line in reader.lines() {
            let line = line?;
            if analyzer.detect_severity(&line) == severity {
                filtered_lines.push(line);
            }
        }

        Ok(filtered_lines)
    }
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_severity_detection() {
        let analyzer = LogAnalyzer::new();
        
        assert_eq!(analyzer.detect_severity("ERROR: Something went wrong"), LogSeverity::Error);
        assert_eq!(analyzer.detect_severity("WARNING: Disk space low"), LogSeverity::Warning);
        assert_eq!(analyzer.detect_severity("DEBUG: Entering function"), LogSeverity::Debug);
        assert_eq!(analyzer.detect_severity("CRITICAL: System failure"), LogSeverity::Critical);
        assert_eq!(analyzer.detect_severity("INFO: Process started"), LogSeverity::Info);
    }

    #[test]
    fn test_file_analysis() -> Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "INFO: Application started")?;
        writeln!(file, "ERROR: Failed to connect")?;
        writeln!(file, "WARNING: High memory usage")?;
        writeln!(file, "INFO: Processing complete")?;
        
        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(file.path())?;
        
        let stats = analyzer.get_statistics();
        assert_eq!(stats.get(&LogSeverity::Info), Some(&2));
        assert_eq!(stats.get(&LogSeverity::Error), Some(&1));
        assert_eq!(analyzer.get_total_lines(), 4);
        
        Ok(())
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
    pub time_range: Option<(String, String)>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_errors: HashMap::new(),
            time_range: None,
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut summary = LogSummary::new();

        for line in reader.lines() {
            let line = line?;
            summary.process_line(&line);
        }

        Ok(summary)
    }

    fn process_line(&mut self, line: &str) {
        self.total_lines += 1;

        let line_lower = line.to_lowercase();
        
        if line_lower.contains("error") {
            self.error_count += 1;
            self.extract_error_pattern(line);
        } else if line_lower.contains("warning") {
            self.warning_count += 1;
        } else if line_lower.contains("info") {
            self.info_count += 1;
        }

        self.extract_timestamp(line);
    }

    fn extract_error_pattern(&mut self, line: &str) {
        let error_key = line
            .split_whitespace()
            .skip_while(|word| !word.to_lowercase().contains("error"))
            .take(3)
            .collect::<Vec<&str>>()
            .join(" ");

        if !error_key.is_empty() {
            *self.unique_errors.entry(error_key).or_insert(0) += 1;
        }
    }

    fn extract_timestamp(&mut self, line: &str) {
        if let Some(first_word) = line.split_whitespace().next() {
            if first_word.contains(':') && first_word.chars().any(|c| c.is_digit(10)) {
                match &mut self.time_range {
                    None => {
                        self.time_range = Some((first_word.to_string(), first_word.to_string()));
                    }
                    Some((start, end)) => {
                        if first_word < *start {
                            *start = first_word.to_string();
                        }
                        if first_word > *end {
                            *end = first_word.to_string();
                        }
                    }
                }
            }
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("=====================");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        
        if let Some((start, end)) = &self.time_range {
            println!("Time range: {} - {}", start, end);
        }
        
        if !self.unique_errors.is_empty() {
            println!("\nUnique error patterns:");
            for (pattern, count) in &self.unique_errors {
                println!("  {}: {}", pattern, count);
            }
        }
        
        let error_percentage = if self.total_lines > 0 {
            (self.error_count as f64 / self.total_lines as f64) * 100.0
        } else {
            0.0
        };
        println!("\nError percentage: {:.2}%", error_percentage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let log_content = r#"2023-10-01 10:00:00 INFO Application started
2023-10-01 10:00:05 ERROR Database connection failed
2023-10-01 10:00:10 WARNING High memory usage detected
2023-10-01 10:00:15 ERROR Database connection failed
2023-10-01 10:00:20 INFO User login successful
2023-10-01 10:00:25 ERROR Invalid input received"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_content).unwrap();
        
        let summary = LogSummary::analyze_file(temp_file.path()).unwrap();
        
        assert_eq!(summary.total_lines, 6);
        assert_eq!(summary.error_count, 3);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 2);
        assert_eq!(summary.unique_errors.len(), 2);
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
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut stats = HashMap::new();
        stats.insert("total_lines".to_string(), 0);
        stats.insert("errors".to_string(), 0);
        stats.insert("warnings".to_string(), 0);
        stats.insert("info".to_string(), 0);
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            
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
        
        let report = analyzer.generate_report(&stats);
        assert!(report.contains("Error rate: 25.00%"));
    }
}