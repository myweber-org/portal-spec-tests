
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            
            if key.is_empty() {
                return Err(format!("Empty key in line: {}", line));
            }

            config.values.insert(key, value);
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        
        for key in required_keys {
            if !self.values.contains_key(*key) {
                missing.push(key.to_string());
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_config() {
        let config = Config::new();
        assert!(config.is_empty());
        assert_eq!(config.len(), 0);
    }

    #[test]
    fn test_valid_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "host=localhost").unwrap();
        writeln!(file, "port=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "timeout=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.len(), 3);
        assert_eq!(config.get("host").unwrap(), "localhost");
        assert_eq!(config.get("port").unwrap(), "8080");
        assert_eq!(config.get("timeout").unwrap(), "30");
        assert_eq!(config.get_or_default("missing", "default"), "default");
    }

    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid_line").unwrap();
        
        let result = Config::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        config.values.insert("host".to_string(), "localhost".to_string());
        config.values.insert("port".to_string(), "8080".to_string());

        let required = vec!["host", "port", "timeout"];
        let result = config.validate_required(&required);
        assert!(result.is_err());
        
        let missing = result.unwrap_err();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "timeout");
    }
}