use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            database_url: String::from("postgresql://localhost:5432/appdb"),
            log_level: String::from("info"),
            cache_size: 1000,
            enable_metrics: true,
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
                eprintln!("Failed to load config: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be zero"));
        }
        if self.cache_size > 100_000 {
            return Err(String::from("Cache size cannot exceed 100,000"));
        }
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level: {}. Must be one of: {:?}",
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
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.enable_metrics);
    }

    #[test]
    fn test_valid_config() {
        let config = AppConfig {
            server_port: 3000,
            database_url: String::from("postgresql://remote:5432/db"),
            log_level: String::from("debug"),
            cache_size: 500,
            enable_metrics: false,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_port() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_log_level() {
        let mut config = AppConfig::default();
        config.log_level = String::from("invalid");
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
            server_port = 9000
            database_url = "postgresql://test:5432/testdb"
            log_level = "warn"
            cache_size = 2000
            enable_metrics = false
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 9000);
        assert_eq!(config.log_level, "warn");
        assert!(!config.enable_metrics);
    }
}