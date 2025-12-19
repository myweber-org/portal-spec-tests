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
    metadata: HashMap<String, String>,
}

struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
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
            let entry: LogEntry = serde_json::from_str(&line)?;
            self.entries.push(entry);
        }
        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    fn group_by_service(&self) -> HashMap<String, Vec<&LogEntry>> {
        let mut groups = HashMap::new();
        for entry in &self.entries {
            groups
                .entry(entry.service.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        groups
    }

    fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn search_in_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LogProcessor::new();
    processor.load_from_file("logs.jsonl")?;

    println!("Total entries: {}", processor.entries.len());

    let error_logs = processor.filter_by_level("ERROR");
    println!("Error logs: {}", error_logs.len());

    let service_groups = processor.group_by_service();
    for (service, logs) in service_groups {
        println!("Service '{}': {} logs", service, logs.len());
    }

    let level_counts = processor.count_by_level();
    for (level, count) in level_counts {
        println!("Level {}: {} occurrences", level, count);
    }

    let search_results = processor.search_in_messages("timeout");
    println!("Found {} logs containing 'timeout'", search_results.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"ERROR","service":"api","message":"Connection timeout","metadata":{"ip":"192.168.1.1"}}
{"timestamp":"2023-10-01T12:01:00Z","level":"INFO","service":"auth","message":"User login successful","metadata":{"user_id":"123"}}
{"timestamp":"2023-10-01T12:02:00Z","level":"WARN","service":"api","message":"High latency detected","metadata":{}}"#;
        temp_file.write_all(log_data.as_bytes()).unwrap();

        let mut processor = LogProcessor::new();
        processor.load_from_file(temp_file.path()).unwrap();

        assert_eq!(processor.entries.len(), 3);
        assert_eq!(processor.filter_by_level("ERROR").len(), 1);
        assert_eq!(processor.search_in_messages("timeout").len(), 1);
        
        let counts = processor.count_by_level();
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("INFO"), Some(&1));
    }
}