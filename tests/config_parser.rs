use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_size: usize,
    pub enable_metrics: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_port: 8080,
            database_url: String::from("postgresql://localhost/app_db"),
            log_level: String::from("info"),
            cache_size: 1000,
            enable_metrics: true,
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be 0"));
        }
        
        if self.database_url.is_empty() {
            return Err(String::from("Database URL cannot be empty"));
        }
        
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        if self.cache_size > 100_000 {
            return Err(String::from("Cache size cannot exceed 100,000"));
        }
        
        Ok(())
    }
    
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.enable_metrics);
    }
    
    #[test]
    fn test_valid_config_parsing() {
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost/test_db"
            log_level = "debug"
            cache_size = 500
            enable_metrics = false
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path());
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "debug");
        assert!(!config.enable_metrics);
    }
    
    #[test]
    fn test_invalid_log_level() {
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost/test_db"
            log_level = "invalid_level"
            cache_size = 500
            enable_metrics = true
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path());
        assert!(config.is_err());
        assert!(config.unwrap_err().contains("Invalid log level"));
    }
    
    #[test]
    fn test_serialization() {
        let config = AppConfig::default();
        let toml_str = config.to_toml();
        assert!(toml_str.is_ok());
        let toml_str = toml_str.unwrap();
        assert!(toml_str.contains("server_port = 8080"));
        assert!(toml_str.contains("log_level = \"info\""));
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
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

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = self.resolve_value(value.trim());
                self.values.insert(key, value);
            }
        }

        Ok(())
    }

    fn resolve_value(&self, raw_value: &str) -> String {
        if raw_value.starts_with('$') {
            let var_name = &raw_value[1..];
            env::var(var_name).unwrap_or_else(|_| raw_value.to_string())
        } else {
            raw_value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values
            .get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut config = ConfigParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "HOST=localhost").unwrap();
        writeln!(temp_file, "PORT=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "TIMEOUT=30").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_environment_substitution() {
        env::set_var("APP_SECRET", "my_secret_key");
        
        let mut config = ConfigParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "SECRET=$APP_SECRET").unwrap();
        writeln!(temp_file, "NORMAL=value").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.get("SECRET"), Some(&"my_secret_key".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"value".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut config = ConfigParser::new();
        config.set("EXISTING", "actual_value");

        assert_eq!(config.get_or_default("EXISTING", "default"), "actual_value");
        assert_eq!(config.get_or_default("MISSING", "default_value"), "default_value");
    }
}