
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line_content) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                    continue;
                }
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect();
        
        Ok(filtered)
    }

    pub fn count_entries(&self) -> Result<usize, ParseError> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

pub fn analyze_logs(file_path: &str) -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new(file_path);
    
    println!("Analyzing logs from: {}", file_path);
    
    let total_entries = parser.count_entries()?;
    println!("Total log entries: {}", total_entries);
    
    let error_logs = parser.filter_by_level("error")?;
    println!("Error entries: {}", error_logs.len());
    
    let warning_logs = parser.filter_by_level("warning")?;
    println!("Warning entries: {}", warning_logs.len());
    
    if !error_logs.is_empty() {
        println!("\nRecent errors:");
        for entry in error_logs.iter().take(5) {
            println!("[{}] {}: {}", entry.timestamp, entry.service, entry.message);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "service": "api", "message": "Server started"}
{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "service": "database", "message": "Connection failed", "metadata": {"retry_count": 3}}
{"timestamp": "2024-01-15T10:32:00Z", "level": "WARNING", "service": "cache", "message": "Memory usage high"}"#;
        
        write!(file, "{}", log_data).unwrap();
        file
    }

    #[test]
    fn test_parse_logs() {
        let file = create_test_log();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();
        
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].service, "database");
    }

    #[test]
    fn test_filter_by_level() {
        let file = create_test_log();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let errors = parser.filter_by_level("error").unwrap();
        
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }

    #[test]
    fn test_count_entries() {
        let file = create_test_log();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let count = parser.count_entries().unwrap();
        
        assert_eq!(count, 3);
    }
}