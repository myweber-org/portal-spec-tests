use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if line.trim().is_empty() {
                continue;
            }
            
            let entry: LogEntry = serde_json::from_str(&line)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            
            self.entries.push(entry);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .cloned()
            .collect()
    }

    pub fn filter_by_service(&self, service: &str) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .cloned()
            .collect()
    }

    pub fn summarize(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
            *summary.entry(entry.service.clone()).or_insert(0) += 1;
        }
        
        summary
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .cloned()
            .collect()
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
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "service": "auth", "message": "Authentication failed", "metadata": {{"user": "john", "ip": "192.168.1.1"}}}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:31:00Z", "level": "INFO", "service": "api", "message": "Request processed", "metadata": {{"endpoint": "/users", "duration": "150ms"}}}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:32:00Z", "level": "ERROR", "service": "auth", "message": "Invalid token", "metadata": {{"user": "jane", "ip": "192.168.1.2"}}}}"#).unwrap();
        file
    }

    #[test]
    fn test_load_logs() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        
        let count = parser.load_from_file(file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(parser.get_entries().len(), 3);
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        let errors = parser.filter_by_level("ERROR");
        assert_eq!(errors.len(), 2);
        
        let infos = parser.filter_by_level("INFO");
        assert_eq!(infos.len(), 1);
    }

    #[test]
    fn test_filter_by_service() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        let auth_logs = parser.filter_by_service("auth");
        assert_eq!(auth_logs.len(), 2);
        
        let api_logs = parser.filter_by_service("api");
        assert_eq!(api_logs.len(), 1);
    }

    #[test]
    fn test_summarize() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        let summary = parser.summarize();
        assert_eq!(summary.get("ERROR"), Some(&2));
        assert_eq!(summary.get("INFO"), Some(&1));
        assert_eq!(summary.get("auth"), Some(&2));
        assert_eq!(summary.get("api"), Some(&1));
    }

    #[test]
    fn test_search_messages() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        let failed_logs = parser.search_messages("failed");
        assert_eq!(failed_logs.len(), 1);
        
        let processed_logs = parser.search_messages("processed");
        assert_eq!(processed_logs.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut parser = LogParser::new();
        let file = create_test_log();
        parser.load_from_file(file.path()).unwrap();
        
        assert_eq!(parser.get_entries().len(), 3);
        parser.clear();
        assert_eq!(parser.get_entries().len(), 0);
    }
}