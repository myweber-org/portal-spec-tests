use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        
        config.apply_env_overrides();
        Ok(config)
    }
    
    fn apply_env_overrides(&mut self) {
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(port) = env::var("APP_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.port = port_num;
            }
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level.to_uppercase();
        }
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        
        if self.port == 0 {
            return Err("Port must be greater than 0".to_string());
        }
        
        let valid_log_levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        Ok(())
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
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
}use std::env;
use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
    pub custom_settings: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let mut config: HashMap<String, String> = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        let database_url = env::var("DATABASE_URL")
            .ok()
            .or_else(|| config.remove("database_url"))
            .ok_or_else(|| "Database URL not found in config or environment".to_string())?;

        let api_key = env::var("API_KEY")
            .ok()
            .or_else(|| config.remove("api_key"))
            .ok_or_else(|| "API key not found in config or environment".to_string())?;

        let debug_mode = env::var("DEBUG_MODE")
            .ok()
            .and_then(|v| v.parse().ok())
            .or_else(|| config.remove("debug_mode").and_then(|v| v.parse().ok()))
            .unwrap_or(false);

        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .or_else(|| config.remove("port").and_then(|v| v.parse().ok()))
            .unwrap_or(8080);

        let custom_settings = config;

        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
            custom_settings,
        })
    }

    pub fn get_custom_setting(&self, key: &str) -> Option<&String> {
        self.custom_settings.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_loading() {
        env::set_var("DATABASE_URL", "postgres://localhost/test");
        env::set_var("API_KEY", "test_key_123");

        let config_content = r#"
            debug_mode = true
            port = 3000
            custom_value = "test"
        "#;

        let temp_file = "test_config.toml";
        fs::write(temp_file, config_content).unwrap();
        env::set_var("CONFIG_PATH", temp_file);

        let config = Config::load().unwrap();

        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.api_key, "test_key_123");
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.port, 3000);
        assert_eq!(config.get_custom_setting("custom_value"), Some(&"test".to_string()));

        fs::remove_file(temp_file).unwrap();
        env::remove_var("DATABASE_URL");
        env::remove_var("API_KEY");
        env::remove_var("CONFIG_PATH");
    }
}