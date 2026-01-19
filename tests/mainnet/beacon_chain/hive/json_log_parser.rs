use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use serde_json::Value;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
    pub metadata: Value,
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

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if let Some(entry) = self.parse_json_entry(&json_value) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_json_entry(&self, json: &Value) -> Option<LogEntry> {
        let timestamp_str = json.get("timestamp")?.as_str()?;
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str).ok()?;
        
        let level = json.get("level")?.as_str()?.to_string();
        let message = json.get("message")?.as_str()?.to_string();
        let metadata = json.get("metadata").cloned().unwrap_or(Value::Null);

        Some(LogEntry {
            timestamp,
            level,
            message,
            metadata,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect();
        
        Ok(filtered)
    }

    pub fn filter_by_time_range(
        &self,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    ) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect();
        
        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_json_logs() {
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00+00:00", "level": "INFO", "message": "System started", "metadata": {"user": "admin"}}
{"timestamp": "2024-01-15T10:35:00+00:00", "level": "ERROR", "message": "Connection failed", "metadata": {"retry_count": 3}}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();
        
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "ERROR");
    }

    #[test]
    fn test_filter_by_level() {
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00+00:00", "level": "INFO", "message": "Test", "metadata": null}
{"timestamp": "2024-01-15T10:31:00+00:00", "level": "ERROR", "message": "Test", "metadata": null}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }
}