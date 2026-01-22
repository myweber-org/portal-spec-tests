use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let processed_value = Self::process_value(value.trim());
                settings.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { settings })
    }

    fn process_value(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next(); // Skip '}'
                        break;
                    }
                    var_name.push(ch);
                    chars.next();
                }
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
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
        writeln!(temp_file, "APP_NAME=MyApplication").unwrap();
        writeln!(temp_file, "VERSION=1.0.0").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "EMPTY_LINE=").unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"MyApplication".to_string()));
        assert_eq!(config.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(config.get("DATABASE_URL"), Some(&"postgres://localhost/db".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "HOST=${DB_HOST}").unwrap();
        writeln!(temp_file, "PORT=5432").unwrap();
        writeln!(temp_file, "CONNECTION=${{NOT_SET}}").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("CONNECTION"), Some(&"".to_string()));
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
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

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                config.values.insert(key, value);
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn merge(&mut self, other: Config) {
        for (key, value) in other.values {
            self.values.insert(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost".to_string()));
        assert_eq!(config.get("API_KEY"), Some("secret123".to_string()));
    }

    #[test]
    fn test_env_override() {
        env::set_var("TEST_KEY", "env_value");
        let mut config = Config::new();
        config.set("TEST_KEY", "file_value");

        assert_eq!(config.get("TEST_KEY"), Some("env_value".to_string()));
    }

    #[test]
    fn test_default_value() {
        let config = Config::new();
        assert_eq!(config.get_with_default("MISSING_KEY", "default_value"), "default_value");
    }
}use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_logging: bool,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_address: String::from("127.0.0.1"),
            port: 8080,
            max_connections: 100,
            enable_logging: true,
            log_level: String::from("info"),
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
        if self.port == 0 {
            return Err("Port cannot be zero".to_string());
        }
        if self.max_connections == 0 {
            return Err("Max connections must be greater than zero".to_string());
        }
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }
        Ok(())
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server_address, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert!(config.enable_logging);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());

        config.port = 8080;
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config.max_connections = 100;
        config.log_level = String::from("invalid");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = config.to_toml().unwrap();
        assert!(toml_str.contains("server_address"));
        assert!(toml_str.contains("port"));
    }

    #[test]
    fn test_config_from_file() {
        let toml_content = r#"
            server_address = "192.168.1.100"
            port = 9000
            max_connections = 250
            enable_logging = false
            log_level = "warn"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_address, "192.168.1.100");
        assert_eq!(config.port, 9000);
        assert_eq!(config.max_connections, 250);
        assert!(!config.enable_logging);
        assert_eq!(config.log_level, "warn");
    }
}