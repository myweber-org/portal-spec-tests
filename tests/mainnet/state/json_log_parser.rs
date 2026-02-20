use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser { entries: Vec::new() }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    self.entries.push(entry);
                    count += 1;
                }
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }

        Ok(count)
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    pub fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service.eq_ignore_ascii_case(service))
            .collect()
    }

    pub fn search_message(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    pub fn get_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }
        summary
    }

    pub fn get_service_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for entry in &self.entries {
            *summary.entry(entry.service.clone()).or_insert(0) += 1;
        }
        summary
    }

    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let log_lines = vec![
            r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","service":"api","message":"Request received","user_id":123}"#,
            r#"{"timestamp":"2023-10-01T12:00:01Z","level":"ERROR","service":"database","message":"Connection failed","attempt":3}"#,
            r#"{"timestamp":"2023-10-01T12:00:02Z","level":"WARN","service":"api","message":"Slow response","duration_ms":1500}"#,
            r#"{"timestamp":"2023-10-01T12:00:03Z","level":"INFO","service":"auth","message":"User logged in","ip":"192.168.1.1"}"#,
        ];

        for line in log_lines {
            writeln!(file, "{}", line).unwrap();
        }

        file
    }

    #[test]
    fn test_load_logs() {
        let file = create_test_log();
        let mut parser = LogParser::new();
        let count = parser.load_from_file(file.path()).unwrap();
        assert_eq!(count, 4);
        assert_eq!(parser.entries.len(), 4);
    }

    #[test]
    fn test_filter_by_level() {
        let file = create_test_log();
        let mut parser = LogParser::new();
        parser.load_from_file(file.path()).unwrap();
        
        let info_logs = parser.filter_by_level("INFO");
        assert_eq!(info_logs.len(), 2);
        
        let error_logs = parser.filter_by_level("ERROR");
        assert_eq!(error_logs.len(), 1);
    }

    #[test]
    fn test_filter_by_service() {
        let file = create_test_log();
        let mut parser = LogParser::new();
        parser.load_from_file(file.path()).unwrap();
        
        let api_logs = parser.filter_by_service("api");
        assert_eq!(api_logs.len(), 2);
    }

    #[test]
    fn test_search_message() {
        let file = create_test_log();
        let mut parser = LogParser::new();
        parser.load_from_file(file.path()).unwrap();
        
        let connection_logs = parser.search_message("connection");
        assert_eq!(connection_logs.len(), 1);
    }

    #[test]
    fn test_summary() {
        let file = create_test_log();
        let mut parser = LogParser::new();
        parser.load_from_file(file.path()).unwrap();
        
        let summary = parser.get_summary();
        assert_eq!(summary.get("INFO"), Some(&2));
        assert_eq!(summary.get("ERROR"), Some(&1));
        assert_eq!(summary.get("WARN"), Some(&1));
    }

    #[test]
    fn test_service_summary() {
        let file = create_test_log();
        let mut parser = LogParser::new();
        parser.load_from_file(file.path()).unwrap();
        
        let service_summary = parser.get_service_summary();
        assert_eq!(service_summary.get("api"), Some(&2));
        assert_eq!(service_summary.get("database"), Some(&1));
        assert_eq!(service_summary.get("auth"), Some(&1));
    }
}