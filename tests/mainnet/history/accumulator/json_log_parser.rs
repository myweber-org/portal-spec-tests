use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct LogFilter {
    pub min_level: Option<LogLevel>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub component_filter: Option<String>,
}

impl LogFilter {
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            if !self.compare_levels(&entry.level, min_level) {
                return false;
            }
        }

        if let Some(start) = &self.start_time {
            if &entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = &self.end_time {
            if &entry.timestamp > end {
                return false;
            }
        }

        if let Some(component) = &self.component_filter {
            if !entry.component.contains(component) {
                return false;
            }
        }

        true
    }

    fn compare_levels(&self, entry_level: &LogLevel, filter_level: &LogLevel) -> bool {
        let level_value = |level: &LogLevel| match level {
            LogLevel::ERROR => 4,
            LogLevel::WARN => 3,
            LogLevel::INFO => 2,
            LogLevel::DEBUG => 1,
            LogLevel::TRACE => 0,
        };

        level_value(entry_level) >= level_value(filter_level)
    }
}

pub struct LogParser {
    filter: LogFilter,
}

impl LogParser {
    pub fn new(filter: LogFilter) -> Self {
        LogParser { filter }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    if self.filter.matches(&entry) {
                        entries.push(entry);
                    }
                }
                Err(e) => eprintln!("Failed to parse log line: {}", e),
            }
        }

        Ok(entries)
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, entries: &[LogEntry], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(entries)?;
        std::fs::write(output_path, json)?;
        Ok(())
    }

    pub fn statistics(&self, entries: &[LogEntry]) -> std::collections::HashMap<LogLevel, usize> {
        let mut stats = std::collections::HashMap::new();
        
        for entry in entries {
            *stats.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_filter_matching() {
        let entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            level: LogLevel::INFO,
            message: "Test message".to_string(),
            component: "api".to_string(),
            metadata: None,
        };

        let filter = LogFilter {
            min_level: Some(LogLevel::INFO),
            start_time: Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()),
            end_time: Some(Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap()),
            component_filter: Some("api".to_string()),
        };

        assert!(filter.matches(&entry));
    }

    #[test]
    fn test_level_comparison() {
        let filter = LogFilter {
            min_level: Some(LogLevel::WARN),
            start_time: None,
            end_time: None,
            component_filter: None,
        };

        assert!(filter.compare_levels(&LogLevel::ERROR, &LogLevel::WARN));
        assert!(filter.compare_levels(&LogLevel::WARN, &LogLevel::WARN));
        assert!(!filter.compare_levels(&LogLevel::INFO, &LogLevel::WARN));
    }
}
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: String,
    filter_fields: HashMap<String, String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            filter_fields: HashMap::new(),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filter_fields.insert(key.to_string(), value.to_string());
    }

    pub fn parse_file(&self, path: &Path) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(line)?;
        
        let timestamp = json["timestamp"]
            .as_str()
            .unwrap_or("")
            .to_string();
            
        let level = json["level"]
            .as_str()
            .unwrap_or("info")
            .to_lowercase();

        if !self.is_level_allowed(&level) {
            return Err("Log level filtered out".into());
        }

        let message = json["message"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if !self.passes_field_filters(&json) {
            return Err("Log entry filtered by field filters".into());
        }

        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = vec!["trace", "debug", "info", "warn", "error", "fatal"];
        
        let min_index = level_order.iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(0);
            
        let entry_index = level_order.iter()
            .position(|&l| l == level)
            .unwrap_or(0);

        entry_index >= min_index
    }

    fn passes_field_filters(&self, json: &Value) -> bool {
        for (key, expected_value) in &self.filter_fields {
            if let Some(actual_value) = json.get(key) {
                if let Some(str_value) = actual_value.as_str() {
                    if str_value != expected_value {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn generate_summary(&self, entries: &[LogEntry]) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        for entry in entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
            
            for key in entry.fields.keys() {
                *summary.entry(format!("field:{}", key)).or_insert(0) += 1;
            }
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
    fn test_parser_with_filters() {
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"User login","user_id":"123","service":"auth"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Database connection failed","service":"db"}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","message":"High memory usage","service":"monitor"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let mut parser = LogParser::new("warn");
        parser.add_filter("service", "monitor");

        let entries = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "warn");
        assert_eq!(entries[0].message, "High memory usage");

        let summary = parser.generate_summary(&entries);
        assert_eq!(summary.get("warn"), Some(&1));
    }
}