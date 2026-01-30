use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub ip_addresses: HashMap<String, usize>,
    pub status_codes: HashMap<u16, usize>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            ip_addresses: HashMap::new(),
            status_codes: HashMap::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut summary = LogSummary::new();

        for line in reader.lines() {
            let line = line?;
            summary.process_line(&line);
        }

        Ok(summary)
    }

    fn process_line(&mut self, line: &str) {
        self.total_lines += 1;

        if line.contains("ERROR") {
            self.error_count += 1;
        } else if line.contains("WARNING") {
            self.warning_count += 1;
        } else if line.contains("INFO") {
            self.info_count += 1;
        }

        if let Some(ip) = extract_ip_address(line) {
            *self.ip_addresses.entry(ip).or_insert(0) += 1;
        }

        if let Some(status) = extract_status_code(line) {
            *self.status_codes.entry(status).or_insert(0) += 1;
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        
        if !self.ip_addresses.is_empty() {
            println!("\nTop IP addresses:");
            let mut ips: Vec<_> = self.ip_addresses.iter().collect();
            ips.sort_by(|a, b| b.1.cmp(a.1));
            for (ip, count) in ips.iter().take(5) {
                println!("  {}: {}", ip, count);
            }
        }

        if !self.status_codes.is_empty() {
            println!("\nStatus codes:");
            let mut codes: Vec<_> = self.status_codes.iter().collect();
            codes.sort_by_key(|&(code, _)| code);
            for (code, count) in codes {
                println!("  {}: {}", code, count);
            }
        }
    }
}

fn extract_ip_address(line: &str) -> Option<String> {
    let ip_pattern = r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b";
    let re = regex::Regex::new(ip_pattern).unwrap();
    
    re.find(line)
        .map(|m| m.as_str().to_string())
}

fn extract_status_code(line: &str) -> Option<u16> {
    let status_pattern = r"\b(\d{3})\b";
    let re = regex::Regex::new(status_pattern).unwrap();
    
    re.captures(line)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_summary_creation() {
        let summary = LogSummary::new();
        assert_eq!(summary.total_lines, 0);
        assert_eq!(summary.error_count, 0);
        assert_eq!(summary.warning_count, 0);
        assert_eq!(summary.info_count, 0);
        assert!(summary.ip_addresses.is_empty());
        assert!(summary.status_codes.is_empty());
    }

    #[test]
    fn test_extract_ip_address() {
        let line = "192.168.1.1 - - [10/Oct/2023:13:55:36 +0000] \"GET /api/data HTTP/1.1\" 200 1234";
        let ip = extract_ip_address(line);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_status_code() {
        let line = "192.168.1.1 - - [10/Oct/2023:13:55:36 +0000] \"GET /api/data HTTP/1.1\" 404 1234";
        let status = extract_status_code(line);
        assert_eq!(status, Some(404));
    }
}