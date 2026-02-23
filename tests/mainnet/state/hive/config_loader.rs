
use serde::Deserialize;
use std::env;
use std::fs;

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
        
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;
        
        if let Ok(port) = env::var("SERVER_PORT") {
            config.server_port = port.parse()?;
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }
        
        config.validate()?;
        Ok(config)
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
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
    fn test_config_load_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost:5432/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        
        std::fs::write(temp_file.path(), config_content).unwrap();
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
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
            cache_ttl: 100,
        };
        
        assert!(invalid_config.validate().is_err());
    }
}use serde::Deserialize;
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
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

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
            return Err("Server port cannot be zero".to_string());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        Ok(())
    }
}

pub fn initialize() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    println!("Configuration loaded successfully: {:?}", config);
    Ok(config)
}use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                config.parse_key_value(key, value);
            }
        }
        
        Ok(config)
    }
    
    fn parse_key_value(&mut self, key: &str, value: &str) {
        match key {
            "DATABASE_URL" => self.database_url = Self::resolve_env_var(value),
            "PORT" => {
                if let Ok(port) = value.parse() {
                    self.port = port;
                }
            }
            "DEBUG_MODE" => self.debug_mode = value.eq_ignore_ascii_case("true"),
            key if key.starts_with("API_KEY_") => {
                let service = key.trim_start_matches("API_KEY_").to_lowercase();
                self.api_keys.insert(service, Self::resolve_env_var(value));
            }
            _ => {}
        }
    }
    
    fn resolve_env_var(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost:5432/app".to_string(),
            port: 8080,
            debug_mode: false,
            api_keys: HashMap::new(),
        }
    }
}