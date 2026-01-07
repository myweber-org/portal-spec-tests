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
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.server_port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }
        
        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!("Invalid log level: {}", self.log_level));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let processed_value = Self::substitute_env_vars(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn substitute_env_vars(input: &str) -> String {
        let mut result = input.to_string();
        for (key, value) in env::vars() {
            let placeholder = format!("${}", key);
            result = result.replace(&placeholder, &value);
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let file_contents = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
        
        let mut config: HashMap<String, String> = toml::from_str(&file_contents)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        Self::apply_env_overrides(&mut config);
        
        Ok(Config {
            database_url: Self::get_value(&config, "database_url")?,
            server_port: Self::get_value(&config, "server_port")?
                .parse()
                .map_err(|e| format!("Invalid server_port: {}", e))?,
            log_level: Self::get_value(&config, "log_level")?,
            cache_ttl: Self::get_value(&config, "cache_ttl")?
                .parse()
                .map_err(|e| format!("Invalid cache_ttl: {}", e))?,
        })
    }
    
    fn apply_env_overrides(config: &mut HashMap<String, String>) {
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config.insert(config_key, value);
            }
        }
    }
    
    fn get_value(config: &HashMap<String, String>, key: &str) -> Result<String, String> {
        config.get(key)
            .cloned()
            .ok_or_else(|| format!("Missing required config key: {}", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_loading() {
        env::set_var("APP_DATABASE_URL", "postgres://test:test@localhost/test");
        
        let config = Config::load();
        assert!(config.is_ok());
        
        if let Ok(cfg) = config {
            assert_eq!(cfg.database_url, "postgres://test:test@localhost/test");
        }
        
        env::remove_var("APP_DATABASE_URL");
    }
}