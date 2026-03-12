use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_json_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line_content = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
        
        if line_content.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line_content)
            .map_err(|e| format!("Line {} JSON parse error: {}", line_num + 1, e))?;

        let entry = LogEntry {
            timestamp: json_value["timestamp"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            level: json_value["level"]
                .as_str()
                .unwrap_or("INFO")
                .to_string(),
            message: json_value["message"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            metadata: json_value["metadata"].clone(),
        };

        entries.push(entry);
    }

    Ok(entries)
}

pub fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level.to_uppercase() == level.to_uppercase())
        .collect()
}

pub fn extract_timestamps(entries: &[LogEntry]) -> Vec<String> {
    entries.iter().map(|entry| entry.timestamp.clone()).collect()
}