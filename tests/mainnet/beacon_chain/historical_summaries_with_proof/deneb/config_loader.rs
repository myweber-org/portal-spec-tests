use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub cache_ttl: u64,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let mut config = if fs::metadata(&config_path).is_ok() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            Self::default_config()
        };

        config.apply_env_overrides();
        config.validate()?;
        
        Ok(config)
    }

    fn default_config() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/app".to_string(),
            server_port: 8080,
            log_level: "info".to_string(),
            cache_ttl: 300,
            features: HashMap::from([
                ("api_rate_limiting".to_string(), true),
                ("cors_enabled".to_string(), false),
            ]),
        }
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server_port = port_num;
            }
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level.to_lowercase();
        }
        
        if let Ok(cache_ttl) = env::var("CACHE_TTL") {
            if let Ok(ttl) = cache_ttl.parse() {
                self.cache_ttl = ttl;
            }
        }
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }
        
        if self.server_port == 0 {
            return Err("Server port must be greater than 0".into());
        }
        
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level).into());
        }
        
        Ok(())
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default_config();
        assert_eq!(config.server_port, 8080);
        assert!(config.is_feature_enabled("api_rate_limiting"));
        assert!(!config.is_feature_enabled("cors_enabled"));
    }
    
    #[test]
    fn test_env_override() {
        env::set_var("DATABASE_URL", "postgresql://prod:5432/db");
        env::set_var("LOG_LEVEL", "DEBUG");
        
        let mut config = Config::default_config();
        config.apply_env_overrides();
        
        assert_eq!(config.database_url, "postgresql://prod:5432/db");
        assert_eq!(config.log_level, "debug");
        
        env::remove_var("DATABASE_URL");
        env::remove_var("LOG_LEVEL");
    }
}