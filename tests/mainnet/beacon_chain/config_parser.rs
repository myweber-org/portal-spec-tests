use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub thresholds: HashMap<String, f64>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        let mut thresholds = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if key.starts_with("threshold_") {
                let num_value: f64 = value
                    .parse()
                    .map_err(|_| format!("Invalid number for key {}: {}", key, value))?;
                thresholds.insert(key, num_value);
            } else {
                settings.insert(key, value);
            }
        }

        Ok(Config {
            settings,
            thresholds,
        })
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_threshold(&self, key: &str) -> Option<&f64> {
        self.thresholds.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Sample config").unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "threshold_cpu=80.5").unwrap();
        writeln!(temp_file, "threshold_memory=90.0").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.get_setting("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get_setting("port"), Some(&"8080".to_string()));
        assert_eq!(config.get_threshold("threshold_cpu"), Some(&80.5));
        assert_eq!(config.get_threshold("threshold_memory"), Some(&90.0));
    }

    #[test]
    fn test_invalid_number() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "threshold_cpu=invalid").unwrap();

        let result = Config::from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}