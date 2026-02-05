use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub enable_cache: bool,
    pub cache_ttl: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/app_db"),
            log_level: String::from("info"),
            enable_cache: true,
            cache_ttl: 300,
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

    pub fn from_file_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}, using defaults", e);
                Self::default()
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be zero"));
        }
        if self.database_url.is_empty() {
            return Err(String::from("Database URL cannot be empty"));
        }
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        if self.cache_ttl > 86400 {
            return Err(String::from("Cache TTL cannot exceed 86400 seconds"));
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
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.enable_cache);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = config.to_toml().unwrap();
        assert!(toml_str.contains("server_port = 8080"));
        assert!(toml_str.contains("log_level = \"info\""));
    }

    #[test]
    fn test_config_from_file() {
        let toml_content = r#"
            server_port = 9090
            database_url = "postgresql://localhost:5432/test_db"
            log_level = "debug"
            enable_cache = false
            cache_ttl = 600
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "debug");
        assert!(!config.enable_cache);
        assert_eq!(config.cache_ttl, 600);
    }
}use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                self.settings.insert(key, value);
            }
        }
        Ok(())
    }

    pub fn get_with_env_fallback(&self, key: &str, env_var: &str) -> Option<String> {
        if let Some(value) = self.settings.get(key) {
            return Some(value.clone());
        }
        
        env::var(env_var).ok()
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.settings
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.settings.contains_key(key)
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
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();
        
        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get_or_default("DATABASE_URL", ""), "postgres://localhost/test");
        assert_eq!(config.get_or_default("API_KEY", ""), "secret123");
        assert!(!config.contains_key("NON_EXISTENT"));
    }

    #[test]
    fn test_env_fallback() {
        env::set_var("TEST_ENV_VAR", "env_value");
        let config = Config::new();
        
        let value = config.get_with_env_fallback("missing_key", "TEST_ENV_VAR");
        assert_eq!(value, Some("env_value".to_string()));
        
        env::remove_var("TEST_ENV_VAR");
    }
}