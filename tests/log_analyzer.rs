use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    fn parse_log_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)")?;

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

                self.entries.push(entry);
                *self.level_counts.entry(level).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    fn generate_summary(&self) {
        println!("Log Analysis Summary");
        println!("====================");
        println!("Total entries: {}", self.entries.len());
        println!("\nLog level distribution:");

        for (level, count) in &self.level_counts {
            let percentage = (*count as f64 / self.entries.len() as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", level, count, percentage);
        }

        if let Some(error_count) = self.level_counts.get("ERROR") {
            if *error_count > 0 {
                println!("\nError entries found:");
                for entry in &self.entries {
                    if entry.level == "ERROR" {
                        println!("  [{}] {}", entry.timestamp, entry.message);
                    }
                }
            }
        }
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
}

fn main() {
    let mut analyzer = LogAnalyzer::new();

    match analyzer.parse_log_file("application.log") {
        Ok(_) => {
            analyzer.generate_summary();
            
            let warnings = analyzer.filter_by_level("WARN");
            if !warnings.is_empty() {
                println!("\nWarning entries ({} total):", warnings.len());
                for entry in warnings.iter().take(5) {
                    println!("  [{}] {}", entry.timestamp, entry.message);
                }
            }
        }
        Err(e) => eprintln!("Failed to parse log file: {}", e),
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LogSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

pub struct LogAnalyzer {
    log_counts: HashMap<LogSeverity, usize>,
    total_lines: usize,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            log_counts: HashMap::new(),
            total_lines: 0,
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.total_lines += 1;
            
            let severity = self.detect_severity(&line);
            *self.log_counts.entry(severity).or_insert(0) += 1;
        }

        Ok(())
    }

    fn detect_severity(&self, line: &str) -> LogSeverity {
        let line_lower = line.to_lowercase();
        
        if line_lower.contains("critical") || line_lower.contains("fatal") {
            LogSeverity::Critical
        } else if line_lower.contains("error") || line_lower.contains("err") {
            LogSeverity::Error
        } else if line_lower.contains("warning") || line_lower.contains("warn") {
            LogSeverity::Warning
        } else if line_lower.contains("debug") {
            LogSeverity::Debug
        } else {
            LogSeverity::Info
        }
    }

    pub fn get_statistics(&self) -> HashMap<LogSeverity, usize> {
        self.log_counts.clone()
    }

    pub fn get_total_lines(&self) -> usize {
        self.total_lines
    }

    pub fn filter_by_severity<P: AsRef<Path>>(
        path: P,
        severity: LogSeverity,
    ) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut filtered_lines = Vec::new();
        let analyzer = LogAnalyzer::new();

        for line in reader.lines() {
            let line = line?;
            if analyzer.detect_severity(&line) == severity {
                filtered_lines.push(line);
            }
        }

        Ok(filtered_lines)
    }
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_severity_detection() {
        let analyzer = LogAnalyzer::new();
        
        assert_eq!(analyzer.detect_severity("ERROR: Something went wrong"), LogSeverity::Error);
        assert_eq!(analyzer.detect_severity("WARNING: Disk space low"), LogSeverity::Warning);
        assert_eq!(analyzer.detect_severity("DEBUG: Entering function"), LogSeverity::Debug);
        assert_eq!(analyzer.detect_severity("CRITICAL: System failure"), LogSeverity::Critical);
        assert_eq!(analyzer.detect_severity("INFO: Process started"), LogSeverity::Info);
    }

    #[test]
    fn test_file_analysis() -> Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "INFO: Application started")?;
        writeln!(file, "ERROR: Failed to connect")?;
        writeln!(file, "WARNING: High memory usage")?;
        writeln!(file, "INFO: Processing complete")?;
        
        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(file.path())?;
        
        let stats = analyzer.get_statistics();
        assert_eq!(stats.get(&LogSeverity::Info), Some(&2));
        assert_eq!(stats.get(&LogSeverity::Error), Some(&1));
        assert_eq!(analyzer.get_total_lines(), 4);
        
        Ok(())
    }
}