use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub defaults: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            defaults: HashMap::from([
                ("timeout".to_string(), "30".to_string()),
                ("retries".to_string(), "3".to_string()),
                ("log_level".to_string(), "info".to_string()),
            ]),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                self.settings.insert(key, value);
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (key, value) in &self.settings {
            match key.as_str() {
                "timeout" | "retries" => {
                    if value.parse::<u32>().is_err() {
                        errors.push(format!("{} must be a positive integer", key));
                    }
                }
                "log_level" => {
                    let valid_levels = ["error", "warn", "info", "debug", "trace"];
                    if !valid_levels.contains(&value.as_str()) {
                        errors.push(format!("{} must be one of: {:?}", key, valid_levels));
                    }
                }
                _ => {}
            }
        }

        errors
    }

    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "timeout=60").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "log_level=debug").unwrap();

        let result = config.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(config.get("timeout"), Some(&"60".to_string()));
        assert_eq!(config.get("log_level"), Some(&"debug".to_string()));
        assert_eq!(config.get("retries"), Some(&"3".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        config.settings.insert("timeout".to_string(), "invalid".to_string());
        config.settings.insert("log_level".to_string(), "unknown".to_string());

        let errors = config.validate();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("timeout must be a positive integer"));
        assert!(errors[1].contains("log_level must be one of:"));
    }
}