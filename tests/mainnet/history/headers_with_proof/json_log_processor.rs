use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    module: String,
    message: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct LogProcessor {
    entries: Vec<LogEntry>,
    stats: HashMap<String, usize>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            stats: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    self.update_stats(&entry);
                    self.entries.push(entry);
                }
                Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
            }
        }

        Ok(())
    }

    fn update_stats(&mut self, entry: &LogEntry) {
        *self.stats.entry(entry.level.clone()).or_insert(0) += 1;
        *self.stats.entry(entry.module.clone()).or_insert(0) += 1;
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn filter_by_module(&self, module: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.module.contains(module))
            .collect()
    }

    pub fn get_statistics(&self) -> &HashMap<String, usize> {
        &self.stats
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, path: P, filtered_entries: Vec<&LogEntry>) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let entries: Vec<&LogEntry> = if filtered_entries.is_empty() {
            self.entries.iter().collect()
        } else {
            filtered_entries
        };
        
        serde_json::to_writer_pretty(file, &entries)?;
        Ok(())
    }
}

pub fn process_log_file(input_path: &str, output_path: &str, level_filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LogProcessor::new();
    processor.load_from_file(input_path)?;

    let filtered = if let Some(level) = level_filter {
        processor.filter_by_level(level)
    } else {
        Vec::new()
    };

    processor.export_to_json(output_path, filtered)?;

    println!("Processing complete.");
    println!("Total entries: {}", processor.entries.len());
    println!("Statistics: {:?}", processor.get_statistics());

    Ok(())
}use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidFormat(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<serde_json::Error> for LogError {
    fn from(err: serde_json::Error) -> Self {
        LogError::ParseError(err)
    }
}

pub struct LogProcessor;

impl LogProcessor {
    pub fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            match serde_json::from_str::<LogEntry>(&line_content) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                    return Err(LogError::InvalidFormat(format!(
                        "Line {}: {}",
                        line_num + 1,
                        e
                    )));
                }
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
        entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn extract_timestamps(entries: &[LogEntry]) -> Vec<&str> {
        entries.iter().map(|entry| entry.timestamp.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","message":"System started"}
{"timestamp":"2023-10-01T12:05:00Z","level":"ERROR","message":"Connection failed","metadata":{"retry_count":3}}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let result = LogProcessor::parse_log_file(temp_file.path());
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "ERROR");
    }

    #[test]
    fn test_filter_logs() {
        let entries = vec![
            LogEntry {
                timestamp: "2023-10-01T12:00:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Test".to_string(),
                metadata: None,
            },
            LogEntry {
                timestamp: "2023-10-01T12:05:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Error".to_string(),
                metadata: None,
            },
        ];

        let errors = LogProcessor::filter_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }
}