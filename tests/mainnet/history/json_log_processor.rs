use serde::{Deserialize, Serialize};
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
    metadata: HashMap<String, String>,
}

struct LogProcessor {
    entries: Vec<LogEntry>,
    stats: HashMap<String, usize>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            stats: HashMap::new(),
        }
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
                Ok(entry) => {
                    self.entries.push(entry);
                }
                Err(e) => eprintln!("Failed to parse line: {}", e),
            }
        }

        self.update_stats();
        Ok(())
    }

    fn update_stats(&mut self) {
        self.stats.clear();
        for entry in &self.entries {
            *self.stats.entry(entry.level.clone()).or_insert(0) += 1;
            *self.stats.entry(entry.service.clone()).or_insert(0) += 1;
        }
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service.eq_ignore_ascii_case(service))
            .collect()
    }

    fn get_summary(&self) -> HashMap<String, usize> {
        self.stats.clone()
    }

    fn export_filtered<P: AsRef<Path>>(
        &self,
        filter: &str,
        value: &str,
        output_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let filtered = match filter.to_lowercase().as_str() {
            "level" => self.filter_by_level(value),
            "service" => self.filter_by_service(value),
            _ => return Err("Invalid filter type".into()),
        };

        let output_file = File::create(output_path)?;
        let mut writer = std::io::BufWriter::new(output_file);

        for entry in filtered {
            let json = serde_json::to_string(entry)?;
            writeln!(writer, "{}", json)?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LogProcessor::new();
    
    processor.load_from_file("logs.jsonl")?;
    
    println!("Total entries: {}", processor.entries.len());
    println!("Statistics: {:?}", processor.get_summary());
    
    let error_logs = processor.filter_by_level("ERROR");
    println!("Error logs count: {}", error_logs.len());
    
    processor.export_filtered("level", "ERROR", "error_logs.jsonl")?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed","metadata":{{"method":"GET","path":"/health"}}}}"#
        ).unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"database","message":"Connection failed","metadata":{{"retry_count":"3"}}}}"#
        ).unwrap();

        let mut processor = LogProcessor::new();
        processor.load_from_file(temp_file.path()).unwrap();

        assert_eq!(processor.entries.len(), 2);
        assert_eq!(processor.filter_by_level("ERROR").len(), 1);
        assert_eq!(processor.filter_by_service("api").len(), 1);
    }
}