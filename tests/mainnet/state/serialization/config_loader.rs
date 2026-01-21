
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
    pub fn new() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
        
        let config: Config = toml::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
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
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut features = HashMap::new();
        features.insert("api_v2".to_string(), false);
        features.insert("caching".to_string(), true);
        features.insert("metrics".to_string(), false);
        
        Self {
            database_url: "postgresql://localhost:5432/mydb".to_string(),
            server_port: 8080,
            log_level: "info".to_string(),
            cache_ttl: 300,
            features,
        }
    }
}