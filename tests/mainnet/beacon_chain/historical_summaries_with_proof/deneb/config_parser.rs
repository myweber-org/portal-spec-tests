use std::fs;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub numeric_values: HashMap<String, f64>,
    pub enabled_features: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            numeric_values: HashMap::new(),
            enabled_features: Vec::new(),
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

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();

                if key.ends_with("_enabled") && value == "true" {
                    let feature_name = key.trim_end_matches("_enabled").to_string();
                    config.enabled_features.push(feature_name);
                } else if let Ok(num) = value.parse::<f64>() {
                    config.numeric_values.insert(key, num);
                } else {
                    config.settings.insert(key, value);
                }
            }
        }

        Ok(config)
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_numeric(&self, key: &str) -> Option<f64> {
        self.numeric_values.get(key).copied()
    }

    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.enabled_features.iter().any(|f| f == feature)
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
        writeln!(temp_file, "# Sample configuration").unwrap();
        writeln!(temp_file, "server_host = localhost").unwrap();
        writeln!(temp_file, "server_port = 8080").unwrap();
        writeln!(temp_file, "timeout = 30.5").unwrap();
        writeln!(temp_file, "logging_enabled = true").unwrap();
        writeln!(temp_file, "cache_enabled = true").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get_setting("server_host"), Some(&"localhost".to_string()));
        assert_eq!(config.get_numeric("server_port"), Some(8080.0));
        assert_eq!(config.get_numeric("timeout"), Some(30.5));
        assert!(config.is_feature_enabled("logging"));
        assert!(config.is_feature_enabled("cache"));
        assert!(!config.is_feature_enabled("monitoring"));
    }
}