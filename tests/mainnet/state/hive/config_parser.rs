use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub defaults: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            defaults: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        self.parse_content(&content)
    }

    pub fn parse_content(&mut self, content: &str) -> Result<(), String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line format: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if value.is_empty() {
                return Err(format!("Empty value for key: {}", key));
            }

            self.settings.insert(key, value);
        }
        Ok(())
    }

    pub fn set_default(&mut self, key: &str, value: &str) {
        self.defaults.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    pub fn validate_required(&self, keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        for key in keys {
            if !self.settings.contains_key(*key) && !self.defaults.contains_key(*key) {
                missing.push(key.to_string());
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut config = Config::new();
        let content = "server_host=127.0.0.1\nserver_port=8080\n# This is a comment\n";
        
        assert!(config.parse_content(content).is_ok());
        assert_eq!(config.get("server_host"), Some(&"127.0.0.1".to_string()));
        assert_eq!(config.get("server_port"), Some(&"8080".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_default_values() {
        let mut config = Config::new();
        config.set_default("timeout", "30");
        
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get_or_default("timeout", "60"), "30");
        assert_eq!(config.get_or_default("nonexistent", "default"), "default");
    }

    #[test]
    fn test_file_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "key1=value1\nkey2=value2").unwrap();
        
        let mut config = Config::new();
        assert!(config.load_from_file(file.path()).is_ok());
        assert_eq!(config.get("key1"), Some(&"value1".to_string()));
        assert_eq!(config.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        config.settings.insert("host".to_string(), "localhost".to_string());
        config.set_default("port", "8080");
        
        assert!(config.validate_required(&["host", "port"]).is_ok());
        
        let result = config.validate_required(&["host", "port", "missing"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["missing".to_string()]);
    }
}