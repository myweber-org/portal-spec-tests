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
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        if !Path::new(&config_path).exists() {
            return Err(format!("Configuration file not found: {}", config_path).into());
        }
        
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;
        
        Self::validate(&mut config)?;
        Self::apply_environment_overrides(&mut config);
        
        Ok(config)
    }
    
    fn validate(config: &mut AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        if config.server_port == 0 {
            return Err("Server port cannot be zero".into());
        }
        
        if config.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&config.log_level.as_str()) {
            config.log_level = "info".to_string();
        }
        
        Ok(())
    }
    
    fn apply_environment_overrides(config: &mut AppConfig) {
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(parsed_port) = port.parse::<u16>() {
                config.server_port = parsed_port;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level.to_lowercase();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost:5432/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        
        std::fs::write(temp_file.path(), config_content).unwrap();
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
        let config = AppConfig::load();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost:5432/mydb");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }
    
    #[test]
    fn test_environment_overrides() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_port = 8080
            database_url = "original_url"
            log_level = "warn"
            cache_ttl = 300
        "#;
        
        std::fs::write(temp_file.path(), config_content).unwrap();
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        env::set_var("SERVER_PORT", "9090");
        env::set_var("DATABASE_URL", "overridden_url");
        
        let config = AppConfig::load().unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.database_url, "overridden_url");
        
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_URL");
    }
}