use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogParser {
    pub filters: Vec<Filter>,
    pub output_format: OutputFormat,
}

#[derive(Debug)]
pub enum Filter {
    Level(String),
    Service(String),
    Contains(String),
    Metadata(String, String),
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Json,
    Text,
    Csv,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: Vec::new(),
            output_format: OutputFormat::Text,
        }
    }

    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    pub fn set_output_format(&mut self, format: OutputFormat) {
        self.output_format = format;
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if let Some(entry) = self.parse_line(&line) {
                if self.passes_filters(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() != 5 {
            return None;
        }

        let mut metadata = HashMap::new();
        let meta_parts: Vec<&str> = parts[4].split(',').collect();
        for meta in meta_parts {
            let kv: Vec<&str> = meta.splitn(2, '=').collect();
            if kv.len() == 2 {
                metadata.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
            }
        }

        Some(LogEntry {
            timestamp: parts[0].trim().to_string(),
            level: parts[1].trim().to_string(),
            service: parts[2].trim().to_string(),
            message: parts[3].trim().to_string(),
            metadata,
        })
    }

    fn passes_filters(&self, entry: &LogEntry) -> bool {
        for filter in &self.filters {
            match filter {
                Filter::Level(level) if entry.level != *level => return false,
                Filter::Service(service) if entry.service != *service => return false,
                Filter::Contains(text) if !entry.message.contains(text) => return false,
                Filter::Metadata(key, value) => {
                    if let Some(v) = entry.metadata.get(key) {
                        if v != value {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                _ => continue,
            }
        }
        true
    }

    pub fn format_entries(&self, entries: &[LogEntry]) -> String {
        match self.output_format {
            OutputFormat::Json => self.format_json(entries),
            OutputFormat::Text => self.format_text(entries),
            OutputFormat::Csv => self.format_csv(entries),
        }
    }

    fn format_json(&self, entries: &[LogEntry]) -> String {
        let mut result = String::from("[");
        for (i, entry) in entries.iter().enumerate() {
            if i > 0 {
                result.push(',');
            }
            result.push_str(&format!(
                r#"{{"timestamp":"{}","level":"{}","service":"{}","message":"{}","metadata":{}}}"#,
                entry.timestamp,
                entry.level,
                entry.service,
                entry.message,
                serde_json::to_string(&entry.metadata).unwrap_or_default()
            ));
        }
        result.push(']');
        result
    }

    fn format_text(&self, entries: &[LogEntry]) -> String {
        let mut result = String::new();
        for entry in entries {
            result.push_str(&format!(
                "[{}] {} {}: {}\n",
                entry.timestamp, entry.level, entry.service, entry.message
            ));
            if !entry.metadata.is_empty() {
                result.push_str(&format!("  Metadata: {:?}\n", entry.metadata));
            }
        }
        result
    }

    fn format_csv(&self, entries: &[LogEntry]) -> String {
        let mut result = String::from("timestamp,level,service,message,metadata\n");
        for entry in entries {
            result.push_str(&format!(
                "{},{},{},{},{:?}\n",
                entry.timestamp,
                entry.level,
                entry.service,
                entry.message,
                entry.metadata
            ));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        let parser = LogParser::new();
        let line = "2024-01-15T10:30:00Z|INFO|auth-service|User login successful|user_id=123,ip=192.168.1.1";
        let entry = parser.parse_line(line).unwrap();

        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.service, "auth-service");
        assert_eq!(entry.message, "User login successful");
        assert_eq!(entry.metadata.get("user_id"), Some(&"123".to_string()));
        assert_eq!(entry.metadata.get("ip"), Some(&"192.168.1.1".to_string()));
    }

    #[test]
    fn test_filter_level() {
        let mut parser = LogParser::new();
        parser.add_filter(Filter::Level("ERROR".to_string()));

        let entry = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: "INFO".to_string(),
            service: "auth-service".to_string(),
            message: "Test".to_string(),
            metadata: HashMap::new(),
        };

        assert!(!parser.passes_filters(&entry));
    }
}use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn extract_errors(&self) -> io::Result<Vec<String>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut errors = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            if line.contains("ERROR") || line.contains("error") {
                errors.push(line);
            }
        }
        
        Ok(errors)
    }
    
    pub fn count_errors(&self) -> io::Result<usize> {
        let errors = self.extract_errors()?;
        Ok(errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_extract_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage").unwrap();
        writeln!(temp_file, "error: File not found").unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.extract_errors().unwrap();
        
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("ERROR"));
        assert!(errors[1].contains("error"));
    }
}use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn extract_errors(&self) -> io::Result<Vec<String>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut errors = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            if line.contains("ERROR") || line.contains("error") {
                errors.push(line);
            }
        }
        
        Ok(errors)
    }
    
    pub fn count_errors(&self) -> io::Result<usize> {
        let errors = self.extract_errors()?;
        Ok(errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_extract_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage").unwrap();
        writeln!(temp_file, "error: File not found").unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.extract_errors().unwrap();
        
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Database connection failed"));
        assert!(errors[1].contains("File not found"));
    }
    
    #[test]
    fn test_count_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ERROR: Test error 1").unwrap();
        writeln!(temp_file, "INFO: Normal operation").unwrap();
        writeln!(temp_file, "ERROR: Test error 2").unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let count = parser.count_errors().unwrap();
        
        assert_eq!(count, 2);
    }
}