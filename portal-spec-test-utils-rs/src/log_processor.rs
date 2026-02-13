use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub struct LogProcessor {
    error_pattern: Regex,
}

impl LogProcessor {
    pub fn new() -> Result<Self, regex::Error> {
        let pattern = r"ERROR|FATAL|CRITICAL|panic|exception";
        let error_regex = Regex::new(pattern)?;
        Ok(LogProcessor {
            error_pattern: error_regex,
        })
    }

    pub fn process_file(&self, file_path: &str) -> io::Result<Vec<String>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            if self.error_pattern.is_match(&line) {
                errors.push(format!("Line {}: {}", line_num + 1, line));
            }
        }

        Ok(errors)
    }

    pub fn count_errors(&self, file_path: &str) -> io::Result<usize> {
        let errors = self.process_file(file_path)?;
        Ok(errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_error_detection() {
        let processor = LogProcessor::new().unwrap();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: System started").unwrap();
        writeln!(temp_file, "ERROR: Disk full").unwrap();
        writeln!(temp_file, "WARNING: High memory usage").unwrap();
        writeln!(temp_file, "FATAL: Kernel panic").unwrap();
        
        let errors = processor.process_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("ERROR: Disk full"));
        assert!(errors[1].contains("FATAL: Kernel panic"));
    }

    #[test]
    fn test_error_count() {
        let processor = LogProcessor::new().unwrap();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        for i in 0..5 {
            writeln!(temp_file, "ERROR: Test error {}", i).unwrap();
        }
        
        let count = processor.count_errors(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 5);
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

impl LogLevel {
    fn from_str(level: &str) -> Option<Self> {
        match level.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warning" => Some(LogLevel::Warning),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
}

pub struct LogProcessor {
    min_level: LogLevel,
    component_filters: Vec<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            component_filters: Vec::new(),
        }
    }

    pub fn add_component_filter(&mut self, component: &str) {
        self.component_filters.push(component.to_string());
    }

    pub fn parse_log_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            if let Some(entry) = self.parse_log_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let timestamp = parts[0].trim().to_string();
        let level_str = parts[1].trim();
        let component = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();

        let level = LogLevel::from_str(level_str)?;

        if !self.should_include(&level, &component) {
            return None;
        }

        Some(LogEntry {
            timestamp,
            level,
            component,
            message,
        })
    }

    fn should_include(&self, level: &LogLevel, component: &str) -> bool {
        let level_priority = match level {
            LogLevel::Error => 4,
            LogLevel::Warning => 3,
            LogLevel::Info => 2,
            LogLevel::Debug => 1,
        };

        let min_priority = match self.min_level {
            LogLevel::Error => 4,
            LogLevel::Warning => 3,
            LogLevel::Info => 2,
            LogLevel::Debug => 1,
        };

        if level_priority < min_priority {
            return false;
        }

        if !self.component_filters.is_empty() && !self.component_filters.contains(&component.to_string()) {
            return false;
        }

        true
    }

    pub fn generate_summary(&self, entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
        let mut summary = HashMap::new();
        
        for entry in entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut processor = LogProcessor::new(LogLevel::Info);
        processor.add_component_filter("network");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 10:30:00 | INFO | network | Connection established").unwrap();
        writeln!(temp_file, "2023-10-01 10:31:00 | DEBUG | network | Packet received").unwrap();
        writeln!(temp_file, "2023-10-01 10:32:00 | ERROR | database | Query failed").unwrap();
        
        let entries = processor.parse_log_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].component, "network");
        assert_eq!(entries[0].level, LogLevel::Info);
    }

    #[test]
    fn test_level_filtering() {
        let processor = LogProcessor::new(LogLevel::Warning);
        
        let entries = vec![
            LogEntry {
                timestamp: "2023-10-01 10:30:00".to_string(),
                level: LogLevel::Info,
                component: "system".to_string(),
                message: "System started".to_string(),
            },
            LogEntry {
                timestamp: "2023-10-01 10:31:00".to_string(),
                level: LogLevel::Error,
                component: "network".to_string(),
                message: "Connection lost".to_string(),
            },
        ];
        
        let summary = processor.generate_summary(&entries);
        assert_eq!(summary.get(&LogLevel::Error), Some(&1));
        assert_eq!(summary.get(&LogLevel::Info), None);
    }
}
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use regex::Regex;

pub struct LogProcessor {
    pattern: Option<Regex>,
    min_level: Option<String>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            pattern: None,
            min_level: None,
        }
    }

    pub fn set_pattern(&mut self, pattern: &str) -> Result<(), regex::Error> {
        self.pattern = Some(Regex::new(pattern)?);
        Ok(())
    }

    pub fn set_min_level(&mut self, level: &str) {
        self.min_level = Some(level.to_lowercase());
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<String>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if self.matches_criteria(&line) {
                results.push(line);
            }
        }

        Ok(results)
    }

    fn matches_criteria(&self, line: &str) -> bool {
        if let Some(ref pattern) = self.pattern {
            if !pattern.is_match(line) {
                return false;
            }
        }

        if let Some(ref min_level) = self.min_level {
            let level_pattern = Regex::new(r"\[(ERROR|WARN|INFO|DEBUG|TRACE)\]").unwrap();
            if let Some(captures) = level_pattern.captures(line) {
                let line_level = captures.get(1).unwrap().as_str().to_lowercase();
                let level_order = |lvl: &str| match lvl {
                    "error" => 0,
                    "warn" => 1,
                    "info" => 2,
                    "debug" => 3,
                    "trace" => 4,
                    _ => 5,
                };

                if level_order(&line_level) > level_order(min_level) {
                    return false;
                }
            }
        }

        true
    }
}

pub fn extract_timestamps(log_lines: &[String]) -> Vec<String> {
    let timestamp_re = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();
    log_lines
        .iter()
        .filter_map(|line| timestamp_re.find(line))
        .map(|m| m.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_filtering() {
        let mut log_data = NamedTempFile::new().unwrap();
        writeln!(log_data, "[INFO] Application started").unwrap();
        writeln!(log_data, "[ERROR] Failed to connect").unwrap();
        writeln!(log_data, "[DEBUG] Processing request").unwrap();

        let mut processor = LogProcessor::new();
        processor.set_min_level("warn").unwrap();

        let results = processor.process_file(log_data.path()).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].contains("INFO"));
        assert!(results[1].contains("ERROR"));
    }

    #[test]
    fn test_pattern_matching() {
        let mut processor = LogProcessor::new();
        processor.set_pattern(r"connection.*failed").unwrap();

        let test_line = "[ERROR] Database connection failed";
        assert!(processor.matches_criteria(test_line));

        let test_line2 = "[INFO] User logged in";
        assert!(!processor.matches_criteria(test_line2));
    }
}