use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogSummary {
    pub total_entries: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub unique_errors: HashMap<String, usize>,
    pub time_range: (String, String),
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_errors: HashMap::new(),
            time_range: (String::new(), String::new()),
        }
    }
}

pub fn analyze_log_file<P: AsRef<Path>>(path: P) -> Result<LogSummary, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut summary = LogSummary::new();
    let mut first_timestamp = String::new();
    let mut last_timestamp = String::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        summary.total_entries += 1;

        if index == 0 {
            first_timestamp = extract_timestamp(&line).unwrap_or_default();
        }
        last_timestamp = extract_timestamp(&line).unwrap_or_default();

        if line.contains("ERROR") {
            summary.error_count += 1;
            let error_key = extract_error_type(&line);
            *summary.unique_errors.entry(error_key).or_insert(0) += 1;
        } else if line.contains("WARNING") {
            summary.warning_count += 1;
        } else if line.contains("INFO") {
            summary.info_count += 1;
        }
    }

    summary.time_range = (first_timestamp, last_timestamp);
    Ok(summary)
}

fn extract_timestamp(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() > 1 {
        Some(parts[0].to_string())
    } else {
        None
    }
}

fn extract_error_type(line: &str) -> String {
    let error_start = line.find("ERROR").unwrap_or(0);
    let error_slice = &line[error_start..];
    let end = error_slice.find(':').unwrap_or(error_slice.len());
    error_slice[..end].to_string()
}

pub fn print_summary(summary: &LogSummary) {
    println!("Log Analysis Summary");
    println!("====================");
    println!("Total entries: {}", summary.total_entries);
    println!("Time range: {} to {}", summary.time_range.0, summary.time_range.1);
    println!("Info entries: {}", summary.info_count);
    println!("Warning entries: {}", summary.warning_count);
    println!("Error entries: {}", summary.error_count);
    
    if !summary.unique_errors.is_empty() {
        println!("\nUnique error types:");
        for (error, count) in &summary.unique_errors {
            println!("  {}: {}", error, count);
        }
    }
    
    let error_percentage = if summary.total_entries > 0 {
        (summary.error_count as f64 / summary.total_entries as f64) * 100.0
    } else {
        0.0
    };
    println!("\nError percentage: {:.2}%", error_percentage);
}