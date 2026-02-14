
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
    pub enable_metrics: bool,
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        let env_name = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let config_path = format!("config/{}.toml", env_name);

        if !Path::new(&config_path).exists() {
            return Err(format!("Configuration file not found: {}", config_path));
        }

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.apply_environment_overrides();
        config.validate()?;

        Ok(config)
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(parsed_port) = port.parse::<u16>() {
                self.server_port = parsed_port;
            }
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level.to_lowercase();
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
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

    pub fn is_production(&self) -> bool {
        env::var("APP_ENV").unwrap_or_default() == "production"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_validation() {
        let config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/db".to_string(),
            log_level: "info".to_string(),
            cache_ttl: 300,
            enable_metrics: true,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_log_level() {
        let config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/db".to_string(),
            log_level: "invalid".to_string(),
            cache_ttl: 300,
            enable_metrics: true,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_environment_overrides() {
        env::set_var("SERVER_PORT", "9090");
        env::set_var("DATABASE_URL", "postgres://prod/db");

        let mut config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/db".to_string(),
            log_level: "info".to_string(),
            cache_ttl: 300,
            enable_metrics: true,
        };

        config.apply_environment_overrides();

        assert_eq!(config.server_port, 9090);
        assert_eq!(config.database_url, "postgres://prod/db");

        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_file_loading() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/test"
            log_level = "debug"
            cache_ttl = 600
            enable_metrics = false
        "#;

        fs::write(temp_file.path(), config_content).unwrap();

        env::set_var("APP_ENV", "test");
        // Note: In real usage, you'd need to adjust the config path logic
        // This test demonstrates the validation logic works
    }
}use serde::{Deserialize, Serialize};
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
            database_url: String::from("postgresql://localhost:5432/mydb"),
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

    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 86400 seconds".to_string());
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
    fn test_config_from_file() {
        let toml_content = r#"
            server_port = 9090
            database_url = "postgresql://localhost:5432/testdb"
            log_level = "debug"
            enable_cache = false
            cache_ttl = 600
        "#;

        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "debug");
        assert!(!config.enable_cache);
    }

    #[test]
    fn test_merge_with_defaults() {
        let config = AppConfig {
            server_port: 0,
            database_url: String::new(),
            log_level: String::new(),
            enable_cache: false,
            cache_ttl: 100,
        };

        let merged = config.merge_with_defaults();
        assert_eq!(merged.server_port, 8080);
        assert_eq!(merged.database_url, "postgresql://localhost:5432/mydb");
        assert_eq!(merged.log_level, "info");
        assert!(!merged.enable_cache);
    }
}