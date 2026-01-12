use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
struct LogSummary {
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    unique_errors: HashMap<String, usize>,
    time_range: (String, String),
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_errors: HashMap::new(),
            time_range: (String::new(), String::new()),
        }
    }
}

fn analyze_log_file(file_path: &str) -> Result<LogSummary, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut summary = LogSummary::new();
    
    let error_regex = Regex::new(r"ERROR")?;
    let warning_regex = Regex::new(r"WARN")?;
    let info_regex = Regex::new(r"INFO")?;
    let timestamp_regex = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}")?;
    
    let mut first_timestamp = String::new();
    let mut last_timestamp = String::new();
    
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        
        if error_regex.is_match(&line) {
            summary.error_count += 1;
            let error_key = extract_error_pattern(&line);
            *summary.unique_errors.entry(error_key).or_insert(0) += 1;
        } else if warning_regex.is_match(&line) {
            summary.warning_count += 1;
        } else if info_regex.is_match(&line) {
            summary.info_count += 1;
        }
        
        if let Some(captures) = timestamp_regex.find(&line) {
            let timestamp = captures.as_str().to_string();
            if line_num == 0 {
                first_timestamp = timestamp.clone();
            }
            last_timestamp = timestamp;
        }
    }
    
    summary.time_range = (first_timestamp, last_timestamp);
    Ok(summary)
}

fn extract_error_pattern(line: &str) -> String {
    let error_pattern_regex = Regex::new(r"ERROR.*?:\s*(.*?)(?:\s+at\s|$)").unwrap();
    
    if let Some(captures) = error_pattern_regex.captures(line) {
        if let Some(error_msg) = captures.get(1) {
            return error_msg.as_str().to_string();
        }
    }
    
    "Unknown error pattern".to_string()
}

fn generate_report(summary: &LogSummary) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("Log Analysis Report\n"));
    report.push_str(&format!("===================\n"));
    report.push_str(&format!("Time Range: {} - {}\n", summary.time_range.0, summary.time_range.1));
    report.push_str(&format!("Total INFO entries: {}\n", summary.info_count));
    report.push_str(&format!("Total WARN entries: {}\n", summary.warning_count));
    report.push_str(&format!("Total ERROR entries: {}\n", summary.error_count));
    
    if !summary.unique_errors.is_empty() {
        report.push_str("\nUnique Error Patterns:\n");
        for (error, count) in &summary.unique_errors {
            report.push_str(&format!("  {} (occurrences: {})\n", error, count));
        }
    }
    
    report
}

fn main() {
    let file_path = "application.log";
    
    match analyze_log_file(file_path) {
        Ok(summary) => {
            let report = generate_report(&summary);
            println!("{}", report);
            
            if summary.error_count > 0 {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to analyze log file: {}", e);
            std::process::exit(1);
        }
    }
}