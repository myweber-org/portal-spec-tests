
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct LogEntry {
    pub ip_address: String,
    pub timestamp: NaiveDateTime,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub response_size: u64,
    pub user_agent: String,
}

pub struct LogParser {
    pattern: Regex,
}

impl LogParser {
    pub fn new() -> Result<Self, regex::Error> {
        let pattern = Regex::new(r#"(?x)
            ^(\S+)                           # IP address
            \s-\s-\s\[([^\]]+)\]            # timestamp
            \s"(\S+)\s(\S+)\s[^"]*"         # method and path
            \s(\d{3})                       # status code
            \s(\d+|-)                       # response size
            \s"[^"]*"\s"([^"]*)"            # user agent
        "#)?;
        
        Ok(LogParser { pattern })
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let captures = self.pattern.captures(line)?;
        
        let ip_address = captures.get(1)?.as_str().to_string();
        
        let timestamp_str = captures.get(2)?.as_str();
        let timestamp = NaiveDateTime::parse_from_str(
            timestamp_str, 
            "%d/%b/%Y:%H:%M:%S %z"
        ).ok()?;
        
        let method = captures.get(3)?.as_str().to_string();
        let path = captures.get(4)?.as_str().to_string();
        
        let status_code = captures.get(5)?.as_str().parse().ok()?;
        
        let response_size = captures.get(6)?.as_str()
            .parse()
            .unwrap_or(0);
        
        let user_agent = captures.get(7)?.as_str().to_string();

        Some(LogEntry {
            ip_address,
            timestamp,
            method,
            path,
            status_code,
            response_size,
            user_agent,
        })
    }

    pub fn parse_file(&self, file_path: &str) -> Result<Vec<LogEntry>, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }

    pub fn analyze_logs(&self, entries: &[LogEntry]) -> HashMap<String, usize> {
        let mut analysis = HashMap::new();
        
        analysis.insert("total_requests".to_string(), entries.len());
        
        let mut status_counts = HashMap::new();
        let mut method_counts = HashMap::new();
        let mut ip_counts = HashMap::new();
        
        for entry in entries {
            *status_counts.entry(entry.status_code).or_insert(0) += 1;
            *method_counts.entry(entry.method.clone()).or_insert(0) += 1;
            *ip_counts.entry(entry.ip_address.clone()).or_insert(0) += 1;
        }
        
        analysis.insert("unique_ips".to_string(), ip_counts.len());
        
        if let Some((most_active_ip, count)) = ip_counts.iter()
            .max_by_key(|&(_, count)| count) 
        {
            analysis.insert("most_active_ip".to_string(), *count);
            analysis.insert("most_active_ip_address".to_string(), 0); // Placeholder
        }
        
        analysis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let parser = LogParser::new().unwrap();
        let line = r#"192.168.1.1 - - [10/Oct/2023:13:55:36 +0000] "GET /api/users HTTP/1.1" 200 1234 "Mozilla/5.0""#;
        
        let entry = parser.parse_line(line).unwrap();
        
        assert_eq!(entry.ip_address, "192.168.1.1");
        assert_eq!(entry.method, "GET");
        assert_eq!(entry.path, "/api/users");
        assert_eq!(entry.status_code, 200);
        assert_eq!(entry.response_size, 1234);
    }

    #[test]
    fn test_parse_invalid_line() {
        let parser = LogParser::new().unwrap();
        let line = "invalid log line";
        
        assert!(parser.parse_line(line).is_none());
    }
}