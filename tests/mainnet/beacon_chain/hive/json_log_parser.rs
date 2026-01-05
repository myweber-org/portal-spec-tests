use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    module: Option<String>,
    thread_id: Option<u32>,
}

struct LogFilter {
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    module_filter: Option<String>,
}

impl LogFilter {
    fn new(min_level: LogLevel) -> Self {
        LogFilter {
            min_level,
            start_time: None,
            end_time: None,
            module_filter: None,
        }
    }

    fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    fn with_module_filter(mut self, module: &str) -> Self {
        self.module_filter = Some(module.to_string());
        self
    }

    fn matches(&self, entry: &LogEntry) -> bool {
        if entry.level as u8 > self.min_level as u8 {
            return false;
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        if let Some(ref module_filter) = self.module_filter {
            if let Some(ref module) = entry.module {
                if module != module_filter {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
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

fn analyze_logs(entries: &[LogEntry]) {
    let mut level_counts = std::collections::HashMap::new();
    let mut module_counts = std::collections::HashMap::new();

    for entry in entries {
        *level_counts.entry(&entry.level).or_insert(0) += 1;
        if let Some(module) = &entry.module {
            *module_counts.entry(module).or_insert(0) += 1;
        }
    }

    println!("Log Analysis:");
    println!("Total entries: {}", entries.len());
    println!("\nBy level:");
    for (level, count) in &level_counts {
        println!("  {:?}: {}", level, count);
    }
    println!("\nBy module:");
    for (module, count) in &module_counts {
        println!("  {}: {}", module, count);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter::new(LogLevel::INFO)
        .with_time_range(
            DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?.with_timezone(&Utc),
            DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")?.with_timezone(&Utc),
        )
        .with_module_filter("database");

    let entries = parse_log_file("logs/app.log", &filter)?;
    
    if entries.is_empty() {
        println!("No log entries match the filter criteria.");
    } else {
        analyze_logs(&entries);
        
        println!("\nSample entries:");
        for entry in entries.iter().take(3) {
            println!("{:?}", entry);
        }
    }

    Ok(())
}use serde::{Deserialize, Serialize};
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
    #[serde(flatten)]
    extra_fields: HashMap<String, serde_json::Value>,
}

struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    fn new() -> Self {
        LogParser { entries: Vec::new() }
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
                Ok(entry) => self.entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
            }
        }

        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    fn get_level_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }
        summary
    }

    fn get_service_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for entry in &self.entries {
            *summary.entry(entry.service.clone()).or_insert(0) += 1;
        }
        summary
    }

    fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    fn export_filtered<P: AsRef<Path>>(
        &self,
        entries: Vec<&LogEntry>,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);

        for entry in entries {
            let json = serde_json::to_string(entry)?;
            writeln!(writer, "{}", json)?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    parser.load_from_file("logs.jsonl")?;

    println!("Total entries loaded: {}", parser.entries.len());

    let error_logs = parser.filter_by_level("error");
    println!("Error logs count: {}", error_logs.len());

    let summary = parser.get_level_summary();
    println!("Level summary: {:?}", summary);

    let service_summary = parser.get_service_summary();
    println!("Service summary: {:?}", service_summary);

    let search_results = parser.search_messages("timeout");
    println!("Found {} entries containing 'timeout'", search_results.len());

    parser.export_filtered(error_logs, "error_logs.jsonl")?;
    println!("Exported error logs to error_logs.jsonl");

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
    #[serde(flatten)]
    extra_fields: serde_json::Value,
}

#[derive(Debug)]
pub struct LogParser {
    min_level: String,
    include_fields: Vec<String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            include_fields: Vec::new(),
        }
    }

    pub fn with_fields(mut self, fields: &[&str]) -> Self {
        self.include_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn Error>> {
        let mut entry: LogEntry = serde_json::from_str(line)?;
        
        if !self.should_include(&entry.level) {
            return Err("Log level below threshold".into());
        }

        if !self.include_fields.is_empty() {
            self.filter_fields(&mut entry);
        }

        Ok(entry)
    }

    fn should_include(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error"];
        let min_idx = level_order.iter().position(|l| *l == self.min_level);
        let entry_idx = level_order.iter().position(|l| *l == level.to_lowercase());

        match (min_idx, entry_idx) {
            (Some(min), Some(entry)) => entry >= min,
            _ => true,
        }
    }

    fn filter_fields(&self, entry: &mut LogEntry) {
        if let serde_json::Value::Object(ref mut map) = entry.extra_fields {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if !self.include_fields.contains(&key) {
                    map.remove(&key);
                }
            }
        }
    }
}

pub fn summarize_logs(entries: &[LogEntry]) -> std::collections::HashMap<String, usize> {
    let mut summary = std::collections::HashMap::new();
    
    for entry in entries {
        *summary.entry(entry.level.clone()).or_insert(0) += 1;
    }
    
    summary
}