
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

pub struct LogProcessor {
    filters: HashMap<LogLevel, bool>,
    min_level: LogLevel,
}

impl LogProcessor {
    pub fn new() -> Self {
        let mut filters = HashMap::new();
        filters.insert(LogLevel::Error, true);
        filters.insert(LogLevel::Warning, true);
        filters.insert(LogLevel::Info, false);
        filters.insert(LogLevel::Debug, false);
        
        LogProcessor {
            filters,
            min_level: LogLevel::Warning,
        }
    }

    pub fn set_filter(&mut self, level: LogLevel, enabled: bool) {
        self.filters.insert(level, enabled);
    }

    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }

    pub fn parse_log_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut filtered_logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if self.should_process_line(&line) {
                filtered_logs.push(line);
            }
        }

        Ok(filtered_logs)
    }

    fn should_process_line(&self, line: &str) -> bool {
        let detected_level = self.detect_log_level(line);
        
        if let Some(level) = detected_level {
            if !self.is_level_enabled(&level) {
                return false;
            }
            
            self.is_level_at_least_min(&level)
        } else {
            false
        }
    }

    fn detect_log_level(&self, line: &str) -> Option<LogLevel> {
        let line_lower = line.to_lowercase();
        
        if line_lower.contains("error") || line_lower.contains("err") {
            Some(LogLevel::Error)
        } else if line_lower.contains("warning") || line_lower.contains("warn") {
            Some(LogLevel::Warning)
        } else if line_lower.contains("info") {
            Some(LogLevel::Info)
        } else if line_lower.contains("debug") {
            Some(LogLevel::Debug)
        } else {
            None
        }
    }

    fn is_level_enabled(&self, level: &LogLevel) -> bool {
        *self.filters.get(level).unwrap_or(&false)
    }

    fn is_level_at_least_min(&self, level: &LogLevel) -> bool {
        let level_value = match level {
            LogLevel::Error => 4,
            LogLevel::Warning => 3,
            LogLevel::Info => 2,
            LogLevel::Debug => 1,
        };
        
        let min_value = match self.min_level {
            LogLevel::Error => 4,
            LogLevel::Warning => 3,
            LogLevel::Info => 2,
            LogLevel::Debug => 1,
        };
        
        level_value >= min_value
    }

    pub fn count_logs_by_level(&self, logs: &[String]) -> HashMap<LogLevel, usize> {
        let mut counts = HashMap::new();
        
        for log in logs {
            if let Some(level) = self.detect_log_level(log) {
                *counts.entry(level).or_insert(0) += 1;
            }
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processor_filters() {
        let mut processor = LogProcessor::new();
        
        let test_logs = vec![
            "ERROR: System failure detected".to_string(),
            "WARNING: High memory usage".to_string(),
            "INFO: User login successful".to_string(),
            "DEBUG: Processing request".to_string(),
        ];
        
        let filtered: Vec<String> = test_logs
            .into_iter()
            .filter(|log| processor.should_process_line(log))
            .collect();
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].contains("ERROR"));
        assert!(filtered[1].contains("WARNING"));
    }

    #[test]
    fn test_parse_log_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ERROR: Disk full").unwrap();
        writeln!(temp_file, "INFO: Backup completed").unwrap();
        writeln!(temp_file, "WARNING: CPU temperature high").unwrap();
        
        let processor = LogProcessor::new();
        let result = processor.parse_log_file(temp_file.path());
        
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_count_logs_by_level() {
        let processor = LogProcessor::new();
        
        let test_logs = vec![
            "ERROR: Failed to connect".to_string(),
            "ERROR: Database timeout".to_string(),
            "WARNING: Slow response".to_string(),
            "INFO: Service started".to_string(),
        ];
        
        let counts = processor.count_logs_by_level(&test_logs);
        
        assert_eq!(counts.get(&LogLevel::Error), Some(&2));
        assert_eq!(counts.get(&LogLevel::Warning), Some(&1));
        assert_eq!(counts.get(&LogLevel::Info), Some(&1));
    }
}