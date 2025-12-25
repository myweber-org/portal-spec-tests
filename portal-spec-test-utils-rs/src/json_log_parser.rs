use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

struct LogParser {
    min_level: String,
    filter_text: Option<String>,
}

impl LogParser {
    fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            filter_text: None,
        }
    }

    fn with_filter(mut self, filter: &str) -> Self {
        self.filter_text = Some(filter.to_lowercase());
        self
    }

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.should_include(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        let entry_level = entry.level.to_lowercase();
        let level_priority = |level: &str| match level {
            "error" => 3,
            "warn" => 2,
            "info" => 1,
            "debug" => 0,
            _ => 0,
        };

        if level_priority(&entry_level) < level_priority(&self.min_level) {
            return false;
        }

        if let Some(filter) = &self.filter_text {
            if !entry.message.to_lowercase().contains(filter) {
                return false;
            }
        }

        true
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("info")
        .with_filter("connection");

    let entries = parser.parse_file("app.log")?;
    
    for entry in entries {
        println!("[{}] {}: {}", 
            entry.timestamp, 
            entry.level.to_uppercase(), 
            entry.message
        );
    }

    Ok(())
}