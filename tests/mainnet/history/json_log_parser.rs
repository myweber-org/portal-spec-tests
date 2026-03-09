use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use serde_json::Value;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
    pub fields: Value,
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
            if let Ok(parsed) = self.parse_line(&line) {
                entries.push(parsed);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp_str = json_value["timestamp"]
            .as_str()
            .ok_or("Missing timestamp field")?;
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;

        let level = json_value["level"]
            .as_str()
            .ok_or("Missing level field")?
            .to_string();

        let message = json_value["message"]
            .as_str()
            .ok_or("Missing message field")?
            .to_string();

        let fields = json_value["fields"].clone();

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let all_entries = self.parse()?;
        let filtered: Vec<LogEntry> = all_entries
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
        let all_entries = self.parse()?;
        let filtered: Vec<LogEntry> = all_entries
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
    fn test_parse_valid_log() {
        let log_data = r#"{"timestamp":"2023-10-05T14:30:00+00:00","level":"INFO","message":"Service started","fields":{"port":8080}}"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[0].message, "Service started");
    }

    #[test]
    fn test_filter_by_level() {
        let log_data = r#"{"timestamp":"2023-10-05T14:30:00+00:00","level":"ERROR","message":"Failed to connect","fields":{}}
{"timestamp":"2023-10-05T14:31:00+00:00","level":"INFO","message":"Connection established","fields":{}}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }

    #[test]
    fn test_filter_by_time_range() {
        let log_data = r#"{"timestamp":"2023-10-05T14:29:00+00:00","level":"INFO","message":"Initializing","fields":{}}
{"timestamp":"2023-10-05T14:30:00+00:00","level":"INFO","message":"Service started","fields":{}}
{"timestamp":"2023-10-05T14:31:00+00:00","level":"INFO","message":"Ready","fields":{}}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let start = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2023, 10, 5, 14, 30, 0).unwrap();
        let end = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2023, 10, 5, 14, 31, 0).unwrap();
        
        let filtered = parser.filter_by_time_range(start, end).unwrap();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| e.timestamp >= start && e.timestamp <= end));
    }
}