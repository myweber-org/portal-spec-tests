use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogFilter {
    min_level: String,
    target_service: Option<String>,
}

impl LogFilter {
    pub fn new(min_level: &str, service: Option<&str>) -> Self {
        LogFilter {
            min_level: min_level.to_lowercase(),
            target_service: service.map(|s| s.to_string()),
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut filtered_logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(parsed) = serde_json::from_str::<Value>(&line) {
                if self.should_include(&parsed) {
                    filtered_logs.push(line);
                }
            }
        }

        Ok(filtered_logs)
    }

    fn should_include(&self, log_entry: &Value) -> bool {
        let level = log_entry.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

        let level_priority = self.get_level_priority(&level);
        let min_priority = self.get_level_priority(&self.min_level);

        if level_priority < min_priority {
            return false;
        }

        if let Some(ref target_service) = self.target_service {
            let service = log_entry.get("service")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if service != target_service {
                return false;
            }
        }

        true
    }

    fn get_level_priority(&self, level: &str) -> u8 {
        match level {
            "error" => 1,
            "warn" => 2,
            "info" => 3,
            "debug" => 4,
            "trace" => 5,
            _ => 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_by_level() {
        let logs = r#"{"level": "ERROR", "message": "System failure", "service": "api"}
{"level": "INFO", "message": "User login", "service": "auth"}
{"level": "DEBUG", "message": "Processing request", "service": "api"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", logs).unwrap();

        let filter = LogFilter::new("info", None);
        let result = filter.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result[0].contains("System failure"));
        assert!(result[1].contains("User login"));
    }

    #[test]
    fn test_filter_by_service() {
        let logs = r#"{"level": "ERROR", "message": "DB error", "service": "database"}
{"level": "WARN", "message": "High latency", "service": "api"}
{"level": "INFO", "message": "Cache miss", "service": "database"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", logs).unwrap();

        let filter = LogFilter::new("warn", Some("database"));
        let result = filter.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].contains("DB error"));
    }
}