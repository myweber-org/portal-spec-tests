use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct LogFilter {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            if !self.level_matches(&entry.level, min_level) {
                return false;
            }
        }

        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if let Ok(entry_time) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                let utc_time = entry_time.with_timezone(&Utc);
                if utc_time < *start || utc_time > *end {
                    return false;
                }
            }
        }

        true
    }

    fn level_matches(&self, entry_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error"];
        let entry_idx = levels.iter().position(|&l| l == entry_level.to_lowercase());
        let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());

        match (entry_idx, min_idx) {
            (Some(e), Some(m)) => e >= m,
            _ => false,
        }
    }
}

fn parse_log_file<P: AsRef<Path>>(path: P, filter: &LogFilter) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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
                if filter.matches(&entry) {
                    entries.push(entry);
                }
            }
            Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
        }
    }

    Ok(entries)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        start_time: Some(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?.with_timezone(&Utc)),
        end_time: Some(DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")?.with_timezone(&Utc)),
    };

    let entries = parse_log_file("application.log", &filter)?;
    
    println!("Found {} log entries matching criteria:", entries.len());
    for entry in entries.iter().take(5) {
        println!("[{:?}] {}: {}", entry.level, entry.timestamp, entry.message);
    }

    Ok(())
}use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
struct LogParser {
    file_path: String,
}

impl LogParser {
    fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
            }
        }

        Ok(entries)
    }

    fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect();
        Ok(filtered)
    }

    fn count_entries(&self) -> Result<usize, Box<dyn Error>> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("application.log");
    
    match parser.parse() {
        Ok(entries) => {
            println!("Total entries parsed: {}", entries.len());
            
            if let Ok(error_logs) = parser.filter_by_level("error") {
                println!("Error entries: {}", error_logs.len());
                for log in error_logs.iter().take(3) {
                    println!("  - {}: {}", log.timestamp, log.message);
                }
            }
        }
        Err(e) => eprintln!("Parser error: {}", e),
    }

    Ok(())
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(flatten)]
    extra_fields: HashMap<String, serde_json::Value>,
}

pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            
            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }
        
        Ok(self.entries.len())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    pub fn get_level_summary(&self) -> HashMap<String, usize> {
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

    pub fn search_messages(&self, query: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&query.to_lowercase()))
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

    fn create_test_log_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        
        let log_lines = vec![
            r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request received","user_id":12345}"#,
            r#"{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"database","message":"Connection failed","attempt":3}"#,
            r#"{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"api","message":"Slow response","duration_ms":1200}"#,
            r#"{"timestamp":"2024-01-15T10:33:00Z","level":"INFO","service":"auth","message":"User logged in","ip":"192.168.1.1"}"#,
        ];
        
        for line in log_lines {
            writeln!(file, "{}", line).unwrap();
        }
        
        file
    }

    #[test]
    fn test_load_logs() {
        let mut parser = LogParser::new();
        let file = create_test_log_file();
        
        let result = parser.load_from_file(file.path());
        assert!(result.is_ok());
        assert_eq!(parser.get_entries().len(), 4);
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        let file = create_test_log_file();
        parser.load_from_file(file.path()).unwrap();
        
        let info_logs = parser.filter_by_level("INFO");
        assert_eq!(info_logs.len(), 2);
        
        let error_logs = parser.filter_by_level("ERROR");
        assert_eq!(error_logs.len(), 1);
    }

    #[test]
    fn test_level_summary() {
        let mut parser = LogParser::new();
        let file = create_test_log_file();
        parser.load_from_file(file.path()).unwrap();
        
        let summary = parser.get_level_summary();
        assert_eq!(summary.get("INFO"), Some(&2));
        assert_eq!(summary.get("ERROR"), Some(&1));
        assert_eq!(summary.get("WARN"), Some(&1));
    }

    #[test]
    fn test_search_messages() {
        let mut parser = LogParser::new();
        let file = create_test_log_file();
        parser.load_from_file(file.path()).unwrap();
        
        let results = parser.search_messages("request");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message, "Request received");
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<HashMap<String, String>>,
}

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    service_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            service_counts: HashMap::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    self.process_entry(entry);
                }
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }

        Ok(())
    }

    fn process_entry(&mut self, entry: LogEntry) {
        *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        *self.service_counts.entry(entry.service.clone()).or_insert(0) += 1;
        self.entries.push(entry);
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service.eq_ignore_ascii_case(service))
            .collect()
    }

    fn generate_summary(&self) -> String {
        let total_entries = self.entries.len();
        let mut summary = format!("Total log entries: {}\n", total_entries);

        summary.push_str("\nLog level distribution:\n");
        for (level, count) in &self.level_counts {
            let percentage = (*count as f64 / total_entries as f64) * 100.0;
            summary.push_str(&format!("  {}: {} ({:.1}%)\n", level, count, percentage));
        }

        summary.push_str("\nService distribution:\n");
        for (service, count) in &self.service_counts {
            let percentage = (*count as f64 / total_entries as f64) * 100.0;
            summary.push_str(&format!("  {}: {} ({:.1}%)\n", service, count, percentage));
        }

        summary
    }

    fn find_errors_with_metadata(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == "ERROR" && entry.metadata.is_some())
            .collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = LogAnalyzer::new();
    
    analyzer.load_from_file("logs.jsonl")?;
    
    println!("{}", analyzer.generate_summary());
    
    let errors = analyzer.filter_by_level("ERROR");
    println!("\nFound {} ERROR entries", errors.len());
    
    let api_entries = analyzer.filter_by_service("api");
    println!("Found {} API service entries", api_entries.len());
    
    let errors_with_metadata = analyzer.find_errors_with_metadata();
    println!("Found {} ERROR entries with metadata", errors_with_metadata.len());
    
    if !errors_with_metadata.is_empty() {
        println!("\nSample error with metadata:");
        let sample = &errors_with_metadata[0];
        println!("Timestamp: {}", sample.timestamp);
        println!("Message: {}", sample.message);
        if let Some(metadata) = &sample.metadata {
            println!("Metadata: {:?}", metadata);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut analyzer = LogAnalyzer::new();
        
        let entry1 = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: "INFO".to_string(),
            service: "api".to_string(),
            message: "Request processed".to_string(),
            metadata: None,
        };
        
        let entry2 = LogEntry {
            timestamp: "2024-01-15T10:31:00Z".to_string(),
            level: "ERROR".to_string(),
            service: "database".to_string(),
            message: "Connection failed".to_string(),
            metadata: Some({
                let mut map = HashMap::new();
                map.insert("error_code".to_string(), "DB_CONN_001".to_string());
                map.insert("retry_count".to_string(), "3".to_string());
                map
            }),
        };
        
        analyzer.process_entry(entry1);
        analyzer.process_entry(entry2);
        
        assert_eq!(analyzer.entries.len(), 2);
        assert_eq!(analyzer.level_counts.get("INFO"), Some(&1));
        assert_eq!(analyzer.level_counts.get("ERROR"), Some(&1));
        assert_eq!(analyzer.service_counts.get("api"), Some(&1));
        assert_eq!(analyzer.service_counts.get("database"), Some(&1));
        
        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        
        let errors_with_metadata = analyzer.find_errors_with_metadata();
        assert_eq!(errors_with_metadata.len(), 1);
    }

    #[test]
    fn test_load_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed","metadata":null}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"database","message":"Connection failed","metadata":{"error_code":"DB_CONN_001"}}"#;
        
        writeln!(temp_file, "{}", log_data).unwrap();
        
        let mut analyzer = LogAnalyzer::new();
        let result = analyzer.load_from_file(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(analyzer.entries.len(), 2);
    }
}