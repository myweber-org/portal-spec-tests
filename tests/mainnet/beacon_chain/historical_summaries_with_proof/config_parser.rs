use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/app_db"),
            log_level: String::from("info"),
            cache_size: 100,
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        if self.cache_size > 10000 {
            return Err("Cache size exceeds maximum limit".to_string());
        }
        Ok(())
    }

    pub fn merge_with_defaults(mut self) -> Self {
        let default = AppConfig::default();
        if self.server_port == 0 {
            self.server_port = default.server_port;
        }
        if self.database_url.is_empty() {
            self.database_url = default.database_url;
        }
        if self.log_level.is_empty() {
            self.log_level = default.log_level;
        }
        if self.cache_size == 0 {
            self.cache_size = default.cache_size;
        }
        self
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
        assert_eq!(config.database_url, "postgresql://localhost:5432/app_db");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_size, 100);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());

        config.server_port = 8080;
        config.database_url = String::new();
        assert!(config.validate().is_err());

        config.database_url = String::from("valid_url");
        config.cache_size = 20000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_merge_with_defaults() {
        let mut config = AppConfig {
            server_port: 0,
            database_url: String::new(),
            log_level: String::new(),
            cache_size: 0,
        };
        config = config.merge_with_defaults();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgresql://localhost:5432/app_db");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_size, 100);
    }

    #[test]
    fn test_from_file() {
        let toml_content = r#"
            server_port = 9000
            database_url = "postgresql://localhost:5432/test_db"
            log_level = "debug"
            cache_size = 500
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 9000);
        assert_eq!(config.database_url, "postgresql://localhost:5432/test_db");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_size, 500);
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut values = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }
        
        Ok(Config { values })
    }
    
    fn process_value(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let env_var = &value[2..value.len() - 1];
            env::var(env_var).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_basic_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DB_HOST=localhost").unwrap();
        writeln!(file, "DB_PASS=${{DB_PASSWORD}}").unwrap();
        writeln!(file, "NORMAL_VALUE=plain_text").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DB_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DB_PASS"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NORMAL_VALUE"), Some(&"plain_text".to_string()));
    }
    
    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default_value"), "default_value");
    }
}