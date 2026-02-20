use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let processed_value = Self::substitute_env_vars(value.trim());
                settings.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { settings })
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        for (key, env_value) in env::vars() {
            let placeholder = format!("${}", key);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &env_value);
            }
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
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
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let mut config: AppConfig = if Path::new(&config_path).exists() {
            let config_content = fs::read_to_string(&config_path)?;
            toml::from_str(&config_content)?
        } else {
            Self::default()
        };

        config.apply_env_overrides();
        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
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
        AppConfig {
            server_port: 8080,
            database_url: "postgresql://localhost:5432/appdb".to_string(),
            log_level: "info".to_string(),
            cache_ttl: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_env_override() {
        env::set_var("SERVER_PORT", "9000");
        env::set_var("LOG_LEVEL", "debug");
        
        let mut config = AppConfig::default();
        config.apply_env_overrides();
        
        assert_eq!(config.server_port, 9000);
        assert_eq!(config.log_level, "debug");
        
        env::remove_var("SERVER_PORT");
        env::remove_var("LOG_LEVEL");
    }
}