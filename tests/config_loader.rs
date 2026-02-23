use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                config_map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(map, "DATABASE_URL")?;
        let port = Self::get_value(map, "PORT")?
            .parse::<u16>()
            .map_err(|e| format!("Invalid port: {}", e))?;
        let log_level = Self::get_value(map, "LOG_LEVEL")?;

        let mut features = HashMap::new();
        if let Some(feature_str) = map.get("FEATURES") {
            for feature in feature_str.split(',') {
                let trimmed = feature.trim();
                if !trimmed.is_empty() {
                    features.insert(trimmed.to_string(), true);
                }
            }
        }

        Ok(Config {
            database_url,
            port,
            log_level,
            features,
        })
    }

    fn get_value(map: &HashMap<String, String>, key: &str) -> Result<String, String> {
        env::var(key)
            .ok()
            .or_else(|| map.get(key).cloned())
            .ok_or_else(|| format!("Missing required configuration: {}", key))
    }

    pub fn feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}use std::env;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub max_connections: u32,
    pub debug_mode: bool,
    pub feature_flags: HashMap<String, bool>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable not set".to_string())?;

        let api_key = env::var("API_KEY")
            .unwrap_or_else(|_| "default_key".to_string());

        let max_connections = env::var("MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid MAX_CONNECTIONS: {}", e))?;

        let debug_mode = env::var("DEBUG_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let mut feature_flags = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with("FEATURE_") {
                let flag_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.parse::<bool>().unwrap_or(false);
                feature_flags.insert(flag_name, enabled);
            }
        }

        Ok(Config {
            database_url,
            api_key,
            max_connections,
            debug_mode,
            feature_flags,
        })
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }

        if self.api_key.len() < 8 {
            errors.push("API key must be at least 8 characters".to_string());
        }

        if self.max_connections == 0 {
            errors.push("Max connections must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.feature_flags
            .get(&feature.to_lowercase())
            .copied()
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        env::set_var("DATABASE_URL", "postgres://localhost/test");
        env::set_var("FEATURE_NEW_API", "true");
        
        let config = Config::load().unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert!(config.is_feature_enabled("new_api"));
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            api_key: "short".to_string(),
            max_connections: 0,
            debug_mode: false,
            feature_flags: HashMap::new(),
        };

        let result = config.validate();
        assert!(result.is_err());
    }
}