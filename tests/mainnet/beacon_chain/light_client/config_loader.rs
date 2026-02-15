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
    pub fn load(config_path: &str) -> Result<Self, String> {
        let path = Path::new(config_path);
        
        if !path.exists() {
            return Err(format!("Configuration file not found: {}", config_path));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, config_path: &str) -> Result<(), String> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(config_path, toml_string)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be zero"));
        }
        
        if self.database_url.is_empty() {
            return Err(String::from("Database URL cannot be empty"));
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        if self.cache_size > 100_000 {
            return Err(String::from("Cache size cannot exceed 100,000"));
        }

        Ok(())
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
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_save_and_load() {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        assert!(config.save(path).is_ok());
        let loaded = AppConfig::load(path).unwrap();
        assert_eq!(config.server_port, loaded.server_port);
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        
        config.apply_env_overrides();
        Ok(config)
    }
    
    fn apply_env_overrides(&mut self) {
        if let Ok(port) = env::var("APP_SERVER_PORT") {
            if let Ok(parsed) = port.parse() {
                self.server_port = parsed;
            }
        }
        
        if let Ok(db_url) = env::var("APP_DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("APP_LOG_LEVEL") {
            self.log_level = log_level;
        }
        
        if let Ok(cache_ttl) = env::var("APP_CACHE_TTL") {
            if let Ok(parsed) = cache_ttl.parse() {
                self.cache_ttl = parsed;
            }
        }
    }
    
    pub fn validate(&self) -> Result<(), String> {
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
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::substitute_env_vars(value.trim());
                settings.insert(key, processed_value);
            }
        }

        Ok(Config { settings })
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                let env_value = env::var(&var_name).unwrap_or_else(|_| String::new());
                result.push_str(&env_value);
            } else {
                result.push(ch);
            }
        }

        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database_url: String::from("postgresql://localhost:5432/mydb"),
            server_port: 8080,
            log_level: String::from("info"),
            cache_ttl: 300,
        }
    }
}

pub struct ConfigLoader {
    config_path: String,
    env_prefix: String,
}

impl ConfigLoader {
    pub fn new(config_path: &str, env_prefix: &str) -> Self {
        ConfigLoader {
            config_path: config_path.to_string(),
            env_prefix: env_prefix.to_string(),
        }
    }

    pub fn load(&self) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let mut config = self.load_from_file()?;
        self.override_from_env(&mut config)?;
        Ok(config)
    }

    fn load_from_file(&self) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let mut file = File::open(&self.config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    fn override_from_env(&self, config: &mut AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        let env_vars: HashMap<String, String> = env::vars()
            .filter(|(k, _)| k.starts_with(&self.env_prefix))
            .collect();

        for (key, value) in env_vars {
            let config_key = key.trim_start_matches(&self.env_prefix).to_lowercase();
            
            match config_key.as_str() {
                "database_url" => config.database_url = value,
                "server_port" => config.server_port = value.parse()?,
                "log_level" => config.log_level = value,
                "cache_ttl" => config.cache_ttl = value.parse()?,
                _ => {}
            }
        }

        Ok(())
    }
}

pub fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.database_url.is_empty() {
        return Err("Database URL cannot be empty".to_string());
    }
    
    if config.server_port == 0 {
        return Err("Server port must be greater than 0".to_string());
    }
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.log_level.as_str()) {
        return Err(format!("Invalid log level: {}", config.log_level));
    }
    
    Ok(())
}