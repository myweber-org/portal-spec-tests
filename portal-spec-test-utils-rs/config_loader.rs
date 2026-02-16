use std::env;
use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let mut config = HashMap::new();
        
        if let Ok(contents) = fs::read_to_string("config.toml") {
            for line in contents.lines() {
                if line.trim().is_empty() || line.starts_with('#') {
                    continue;
                }
                
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    config.insert(
                        parts[0].trim().to_string(),
                        parts[1].trim().to_string()
                    );
                }
            }
        }
        
        let database_url = env::var("DATABASE_URL")
            .ok()
            .or_else(|| config.get("database_url").cloned())
            .ok_or("Database URL not found in config or environment")?;
            
        let api_key = env::var("API_KEY")
            .ok()
            .or_else(|| config.get("api_key").cloned())
            .ok_or("API key not found in config or environment")?;
            
        let debug_mode = env::var("DEBUG_MODE")
            .ok()
            .and_then(|v| v.parse().ok())
            .or_else(|| config.get("debug_mode").and_then(|v| v.parse().ok()))
            .unwrap_or(false);
            
        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .or_else(|| config.get("port").and_then(|v| v.parse().ok()))
            .unwrap_or(8080);
            
        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
        })
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }
        
        if self.api_key.len() < 32 {
            errors.push("API key must be at least 32 characters".to_string());
        }
        
        if self.port == 0 || self.port > 65535 {
            errors.push(format!("Port {} is invalid", self.port));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        if Path::new(&config_path).exists() {
            let config_content = fs::read_to_string(&config_path)?;
            let mut config: AppConfig = toml::from_str(&config_content)?;
            
            config.apply_environment_overrides();
            Ok(config)
        } else {
            let default_config = Self::default();
            let toml_content = toml::to_string_pretty(&default_config)?;
            fs::write(&config_path, toml_content)?;
            Ok(default_config)
        }
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(parsed_port) = port.parse() {
                self.server_port = parsed_port;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
        
        if let Ok(cache_ttl) = env::var("CACHE_TTL") {
            if let Ok(parsed_ttl) = cache_ttl.parse() {
                self.cache_ttl = parsed_ttl;
            }
        }
    }
    
    fn default() -> Self {
        Self {
            server_port: 8080,
            database_url: "postgresql://localhost/app_db".to_string(),
            log_level: "info".to_string(),
            cache_ttl: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let config_content = r#"
            server_port = 3000
            database_url = "postgresql://test/db"
            log_level = "debug"
            cache_ttl = 1800
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();
        
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
        let config = AppConfig::load().unwrap();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.database_url, "postgresql://test/db");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_ttl, 1800);
        
        env::remove_var("CONFIG_PATH");
    }
    
    #[test]
    fn test_environment_overrides() {
        env::set_var("SERVER_PORT", "9000");
        env::set_var("DATABASE_URL", "postgresql://prod/db");
        
        let config_content = r#"
            server_port = 3000
            database_url = "postgresql://test/db"
            log_level = "debug"
            cache_ttl = 1800
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();
        
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
        let config = AppConfig::load().unwrap();
        assert_eq!(config.server_port, 9000);
        assert_eq!(config.database_url, "postgresql://prod/db");
        
        env::remove_var("CONFIG_PATH");
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_URL");
    }
}