use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warn_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warn_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut stats = HashMap::new();
        stats.insert("total_lines".to_string(), 0);
        stats.insert("errors".to_string(), 0);
        stats.insert("warnings".to_string(), 0);
        stats.insert("info".to_string(), 0);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| e.to_string())?;
            
            *stats.get_mut("total_lines").unwrap() += 1;
            
            if self.error_pattern.is_match(&line) {
                *stats.get_mut("errors").unwrap() += 1;
            } else if self.warn_pattern.is_match(&line) {
                *stats.get_mut("warnings").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.get_mut("info").unwrap() += 1;
            }
        }

        Ok(stats)
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let total = stats.get("total_lines").unwrap_or(&0);
        let errors = stats.get("errors").unwrap_or(&0);
        let warnings = stats.get("warnings").unwrap_or(&0);
        let info = stats.get("info").unwrap_or(&0);
        
        format!(
            "Log Analysis Report:\n\
            Total lines: {}\n\
            Errors: {}\n\
            Warnings: {}\n\
            Info messages: {}",
            total, errors, warnings, info
        )
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
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: Processing complete").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(*stats.get("total_lines").unwrap(), 4);
        assert_eq!(*stats.get("errors").unwrap(), 1);
        assert_eq!(*stats.get("warnings").unwrap(), 1);
        assert_eq!(*stats.get("info").unwrap(), 2);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use chrono::{DateTime, Utc};

pub struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    module: String,
    message: String,
    metadata: HashMap<String, String>,
}

impl LogEntry {
    pub fn new(timestamp: DateTime<Utc>, level: &str, module: &str, message: &str) -> Self {
        LogEntry {
            timestamp,
            level: level.to_string(),
            module: module.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn is_error(&self) -> bool {
        self.level == "ERROR" || self.level == "FATAL"
    }
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    error_count: usize,
    warning_count: usize,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn parse_log_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z)\s+(\w+)\s+\[(\w+)\]\s+(.+)$")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp_str = captures.get(1).unwrap().as_str();
                let level = captures.get(2).unwrap().as_str();
                let module = captures.get(3).unwrap().as_str();
                let message = captures.get(4).unwrap().as_str();

                let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?.with_timezone(&Utc);
                let mut entry = LogEntry::new(timestamp, level, module, message);

                if entry.is_error() {
                    self.error_count += 1;
                } else if level == "WARN" {
                    self.warning_count += 1;
                }

                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_module(&self, module: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.module == module)
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), self.entries.len());
        stats.insert("errors".to_string(), self.error_count);
        stats.insert("warnings".to_string(), self.warning_count);

        let mut module_counts = HashMap::new();
        for entry in &self.entries {
            *module_counts.entry(entry.module.clone()).or_insert(0) += 1;
        }

        for (module, count) in module_counts {
            stats.insert(format!("module_{}", module), count);
        }

        stats
    }

    pub fn find_pattern(&self, pattern: &str) -> Result<Vec<&LogEntry>, regex::Error> {
        let re = Regex::new(pattern)?;
        Ok(self.entries
            .iter()
            .filter(|entry| re.is_match(&entry.message))
            .collect())
    }
}

pub fn analyze_logs(path: &str) -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
    let mut analyzer = LogAnalyzer::new();
    analyzer.parse_log_file(path)?;
    Ok(analyzer.get_statistics())
}