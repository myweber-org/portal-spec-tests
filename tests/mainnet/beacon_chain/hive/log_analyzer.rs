use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    source_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            source_counts: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.parse_line(&line);
        }

        Ok(())
    }

    fn parse_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() == 4 {
            let entry = LogEntry {
                timestamp: parts[0].trim().to_string(),
                level: parts[1].trim().to_string(),
                source: parts[2].trim().to_string(),
                message: parts[3].trim().to_string(),
            };

            *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
            *self.source_counts.entry(entry.source.clone()).or_insert(0) += 1;
            self.entries.push(entry);
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_source(&self, source: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.source == source)
            .collect()
    }

    pub fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let mut summary = format!("Total log entries: {}\n", total_entries);

        summary.push_str("\nLevel distribution:\n");
        for (level, count) in &self.level_counts {
            let percentage = (*count as f64 / total_entries as f64) * 100.0;
            summary.push_str(&format!("  {}: {} ({:.1}%)\n", level, count, percentage));
        }

        summary.push_str("\nSource distribution:\n");
        for (source, count) in &self.source_counts {
            let percentage = (*count as f64 / total_entries as f64) * 100.0;
            summary.push_str(&format!("  {}: {} ({:.1}%)\n", source, count, percentage));
        }

        summary
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
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
    fn test_log_analyzer() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 10:00:00 | INFO | server | Application started").unwrap();
        writeln!(temp_file, "2023-10-01 10:05:00 | ERROR | database | Connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 10:10:00 | WARN | server | High memory usage").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.filter_by_source("server").len(), 2);
        
        let summary = analyzer.get_summary();
        assert!(summary.contains("Total log entries: 3"));
        assert!(summary.contains("INFO"));
        assert!(summary.contains("ERROR"));
        assert!(summary.contains("WARN"));
    }
}