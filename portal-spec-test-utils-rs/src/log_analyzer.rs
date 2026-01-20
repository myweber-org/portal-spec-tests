use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    module: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    module_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            module_counts: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.add_entry(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 4 {
            return None;
        }

        Some(LogEntry {
            timestamp: parts[0].to_string(),
            level: parts[1].to_string(),
            module: parts[2].to_string(),
            message: parts[3].to_string(),
        })
    }

    fn add_entry(&mut self, entry: LogEntry) {
        *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        *self.module_counts.entry(entry.module.clone()).or_insert(0) += 1;
        self.entries.push(entry);
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_module(&self, module: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.module == module)
            .collect()
    }

    pub fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let level_summary: Vec<String> = self
            .level_counts
            .iter()
            .map(|(level, count)| format!("{}: {}", level, count))
            .collect();
        let module_summary: Vec<String> = self
            .module_counts
            .iter()
            .map(|(module, count)| format!("{}: {}", module, count))
            .collect();

        format!(
            "Total entries: {}\nLevels: {}\nModules: {}",
            total_entries,
            level_summary.join(", "),
            module_summary.join(", ")
        )
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01T10:00:00 INFO network Connected to server").unwrap();
        writeln!(temp_file, "2023-10-01T10:01:00 ERROR database Connection failed").unwrap();
        writeln!(temp_file, "2023-10-01T10:02:00 WARN network Timeout detected").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.filter_by_module("network").len(), 2);
        
        let summary = analyzer.get_summary();
        assert!(summary.contains("Total entries: 3"));
        assert!(summary.contains("INFO: 1"));
        assert!(summary.contains("network: 2"));
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

    pub fn analyze_log_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();
        
        for line in reader.lines() {
            let line_content = line.map_err(|e| format!("Failed to read line: {}", e))?;
            self.analyze_line(&line_content, &mut stats);
        }
        
        Ok(stats)
    }
    
    fn analyze_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("errors".to_string()).or_insert(0) += 1;
        } else if self.warning_pattern.is_match(line) {
            *stats.entry("warnings".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("info_messages".to_string()).or_insert(0) += 1;
        }
    }
    
    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::from("Log Analysis Report\n");
        report.push_str("===================\n");
        
        for (category, count) in stats {
            report.push_str(&format!("{}: {}\n", category, count));
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
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 INFO: Application started").unwrap();
        writeln!(temp_file, "2023-10-01 ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 WARN: High memory usage detected").unwrap();
        writeln!(temp_file, "2023-10-01 INFO: User login successful").unwrap();
        
        let stats = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("errors"), Some(&1));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("info_messages"), Some(&2));
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
    pub ip_addresses: HashMap<String, usize>,
    pub status_codes: HashMap<u16, usize>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            ip_addresses: HashMap::new(),
            status_codes: HashMap::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
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

        if line.contains("[ERROR]") {
            self.error_count += 1;
        } else if line.contains("[WARN]") {
            self.warning_count += 1;
        } else if line.contains("[INFO]") {
            self.info_count += 1;
        }

        self.extract_ip_address(line);
        self.extract_status_code(line);
    }

    fn extract_ip_address(&mut self, line: &str) {
        let ip_pattern = r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b";
        if let Some(captures) = regex::Regex::new(ip_pattern).unwrap().find(line) {
            let ip = captures.as_str().to_string();
            *self.ip_addresses.entry(ip).or_insert(0) += 1;
        }
    }

    fn extract_status_code(&mut self, line: &str) {
        let status_pattern = r"\bHTTP/\d\.\d\"\s+(\d{3})\b";
        if let Some(captures) = regex::Regex::new(status_pattern).unwrap().captures(line) {
            if let Some(status_str) = captures.get(1) {
                if let Ok(status_code) = status_str.as_str().parse::<u16>() {
                    *self.status_codes.entry(status_code).or_insert(0) += 1;
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
        
        println!("\nTop IP addresses:");
        let mut ip_vec: Vec<(&String, &usize)> = self.ip_addresses.iter().collect();
        ip_vec.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in ip_vec.iter().take(5) {
            println!("  {}: {}", ip, count);
        }

        println!("\nHTTP Status codes:");
        let mut status_vec: Vec<(&u16, &usize)> = self.status_codes.iter().collect();
        status_vec.sort_by_key(|&(code, _)| code);
        for (code, count) in status_vec {
            println!("  {}: {}", code, count);
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
        let log_content = r#"127.0.0.1 - - [10/Oct/2023:13:55:36] "GET /api/users HTTP/1.1" 200 1234 [INFO] User login successful
192.168.1.100 - - [10/Oct/2023:13:55:37] "POST /api/data HTTP/1.1" 404 567 [ERROR] Resource not found
10.0.0.1 - - [10/Oct/2023:13:55:38] "GET /api/status HTTP/1.1" 500 789 [WARN] Server error
127.0.0.1 - - [10/Oct/2023:13:55:39] "GET /api/health HTTP/1.1" 200 456 [INFO] Health check passed"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_content).unwrap();
        
        let summary = LogSummary::analyze_file(temp_file.path()).unwrap();
        
        assert_eq!(summary.total_lines, 4);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 2);
        assert_eq!(summary.ip_addresses.get("127.0.0.1"), Some(&2));
        assert_eq!(summary.status_codes.get(&200), Some(&2));
        assert_eq!(summary.status_codes.get(&404), Some(&1));
        assert_eq!(summary.status_codes.get(&500), Some(&1));
    }
}