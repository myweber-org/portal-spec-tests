use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warning_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warning_pattern: Regex::new(r"WARNING").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_log_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            
            if self.error_pattern.is_match(&line) {
                *stats.entry("errors".to_string()).or_insert(0) += 1;
            } else if self.warning_pattern.is_match(&line) {
                *stats.entry("warnings".to_string()).or_insert(0) += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.entry("info".to_string()).or_insert(0) += 1;
            }
        }
        
        Ok(stats)
    }
    
    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::new();
        report.push_str("Log Analysis Report\n");
        report.push_str("===================\n");
        
        for (level, count) in stats {
            report.push_str(&format!("{}: {}\n", level, count));
        }
        
        let total: usize = stats.values().sum();
        report.push_str(&format!("\nTotal log entries analyzed: {}", total));
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2024-01-15 INFO: Application started").unwrap();
        writeln!(temp_file, "2024-01-15 WARNING: Low memory").unwrap();
        writeln!(temp_file, "2024-01-15 ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "2024-01-15 INFO: User logged in").unwrap();
        
        let stats = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("info"), Some(&2));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("errors"), Some(&1));
        
        let report = analyzer.generate_report(&stats);
        assert!(report.contains("info: 2"));
        assert!(report.contains("Total log entries analyzed: 4"));
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp = captures[1].to_string();
                let level = captures[2].to_string();
                let message = captures[3].to_string();

                let entry = LogEntry {
                    timestamp,
                    level: level.clone(),
                    message,
                };

                *self.level_counts.entry(level).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn get_level_summary(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "2023-10-01 12:00:00 [INFO] Application started"
        ).unwrap();
        writeln!(
            temp_file,
            "2023-10-01 12:01:00 [ERROR] Failed to connect to database"
        ).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.total_entries(), 2);
        assert_eq!(analyzer.get_level_summary().get("INFO"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("ERROR"), Some(&1));
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    endpoint: String,
    status_code: u16,
    response_time: u64,
}

struct LogAnalyzer {
    total_requests: u64,
    error_requests: u64,
    endpoint_stats: HashMap<String, EndpointStats>,
}

#[derive(Debug)]
struct EndpointStats {
    count: u64,
    total_time: u64,
    errors: u64,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            total_requests: 0,
            error_requests: 0,
            endpoint_stats: HashMap::new(),
        }
    }

    fn process_log(&mut self, entry: &LogEntry) {
        self.total_requests += 1;
        
        if entry.status_code >= 400 {
            self.error_requests += 1;
        }

        let stats = self.endpoint_stats
            .entry(entry.endpoint.clone())
            .or_insert_with(|| EndpointStats {
                count: 0,
                total_time: 0,
                errors: 0,
            });

        stats.count += 1;
        stats.total_time += entry.response_time;
        
        if entry.status_code >= 400 {
            stats.errors += 1;
        }
    }

    fn calculate_error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.error_requests as f64 / self.total_requests as f64) * 100.0
    }

    fn get_top_endpoints(&self, limit: usize) -> Vec<(String, EndpointStats)> {
        let mut endpoints: Vec<_> = self.endpoint_stats.iter().collect();
        endpoints.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        endpoints.into_iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), EndpointStats {
                count: v.count,
                total_time: v.total_time,
                errors: v.errors,
            }))
            .collect()
    }

    fn get_average_response_time(&self) -> HashMap<String, f64> {
        self.endpoint_stats.iter()
            .map(|(endpoint, stats)| {
                let avg_time = if stats.count > 0 {
                    stats.total_time as f64 / stats.count as f64
                } else {
                    0.0
                };
                (endpoint.clone(), avg_time)
            })
            .collect()
    }
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let re = Regex::new(r"\[(.*?)\] \"(.*?)\" (\d{3}) (\d+)ms").unwrap();
    
    if let Some(captures) = re.captures(line) {
        Some(LogEntry {
            timestamp: captures[1].to_string(),
            endpoint: captures[2].to_string(),
            status_code: captures[3].parse().unwrap_or(0),
            response_time: captures[4].parse().unwrap_or(0),
        })
    } else {
        None
    }
}

fn analyze_log_file(file_path: &str) -> Result<LogAnalyzer, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut analyzer = LogAnalyzer::new();

    for line in reader.lines() {
        if let Ok(line_content) = line {
            if let Some(log_entry) = parse_log_line(&line_content) {
                analyzer.process_log(&log_entry);
            }
        }
    }

    Ok(analyzer)
}

fn main() {
    let file_path = "server.log";
    
    match analyze_log_file(file_path) {
        Ok(analyzer) => {
            println!("Total requests: {}", analyzer.total_requests);
            println!("Error rate: {:.2}%", analyzer.calculate_error_rate());
            
            println!("\nTop 5 endpoints:");
            for (endpoint, stats) in analyzer.get_top_endpoints(5) {
                println!("  {}: {} requests ({} errors)", 
                    endpoint, stats.count, stats.errors);
            }
            
            println!("\nAverage response times:");
            for (endpoint, avg_time) in analyzer.get_average_response_time() {
                println!("  {}: {:.2}ms", endpoint, avg_time);
            }
        }
        Err(e) => {
            eprintln!("Error analyzing log file: {}", e);
        }
    }
}