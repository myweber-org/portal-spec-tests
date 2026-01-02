use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u32,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        if !Path::new(&config_path).exists() {
            return Err("Configuration file not found".into());
        }

        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;

        if let Ok(env_port) = env::var("SERVER_PORT") {
            if let Ok(port) = env_port.parse() {
                config.server_port = port;
            }
        }

        if let Ok(env_db_url) = env::var("DATABASE_URL") {
            config.database_url = env_db_url;
        }

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".into());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours".into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_load_default() {
        let config_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost:5432/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        fs::write(config_file.path(), config_content).unwrap();

        env::set_var("CONFIG_PATH", config_file.path().to_str().unwrap());
        let config = AppConfig::load().unwrap();

        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost:5432/mydb");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_config_validation() {
        let invalid_config = AppConfig {
            server_port: 0,
            database_url: "".to_string(),
            log_level: "invalid".to_string(),
            cache_ttl: 100000,
        };

        let result = invalid_config.validate();
        assert!(result.is_err());
    }
}