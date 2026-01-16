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
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: NaiveDateTime,
    level: String,
    component: String,
    message: String,
    metadata: HashMap<String, String>,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    error_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            error_pattern: Regex::new(r"ERROR|FATAL|CRITICAL").unwrap(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> std::io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some(entry) = self.parse_log_line(&line) {
                    self.entries.push(entry);
                }
            }
        }
        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 {
            return None;
        }

        let timestamp_str = parts[0].trim();
        let level = parts[1].trim().to_string();
        let component = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();

        let timestamp = match NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S") {
            Ok(ts) => ts,
            Err(_) => return None,
        };

        let mut metadata = HashMap::new();
        if let Some(meta_start) = message.find('{') {
            let meta_str = &message[meta_start..];
            if let Ok(meta_map) = self.parse_metadata(meta_str) {
                metadata = meta_map;
            }
        }

        Some(LogEntry {
            timestamp,
            level,
            component,
            message,
            metadata,
        })
    }

    fn parse_metadata(&self, meta_str: &str) -> Result<HashMap<String, String>, ()> {
        let mut map = HashMap::new();
        let clean_str = meta_str.trim_matches(|c| c == '{' || c == '}');
        
        for pair in clean_str.split(',') {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                map.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
            }
        }
        Ok(map)
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn find_errors(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| self.error_pattern.is_match(&entry.level))
            .collect()
    }

    pub fn get_component_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for entry in &self.entries {
            *stats.entry(entry.component.clone()).or_insert(0) += 1;
        }
        stats
    }

    pub fn time_range_entries(&self, start: NaiveDateTime, end: NaiveDateTime) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_log_parsing() {
        let mut analyzer = LogAnalyzer::new();
        let test_line = "2024-01-15 14:30:00 | INFO | network | Connection established {ip=192.168.1.1, port=8080}";
        
        if let Some(entry) = analyzer.parse_log_line(test_line) {
            assert_eq!(entry.level, "INFO");
            assert_eq!(entry.component, "network");
            assert!(entry.metadata.contains_key("ip"));
        } else {
            panic!("Failed to parse log line");
        }
    }

    #[test]
    fn test_error_filtering() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: NaiveDate::from_ymd(2024, 1, 15).and_hms(10, 0, 0),
            level: "ERROR".to_string(),
            component: "database".to_string(),
            message: "Connection failed".to_string(),
            metadata: HashMap::new(),
        });

        let errors = analyzer.find_errors();
        assert_eq!(errors.len(), 1);
    }
}