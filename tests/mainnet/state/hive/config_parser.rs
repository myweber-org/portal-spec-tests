use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub features: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config_map.insert(key, value);
            }
        }

        Self::from_map(&config_map)
    }

    pub fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(map, "DATABASE_URL")
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/app".to_string());

        let port = Self::get_value(map, "PORT")
            .or_else(|| env::var("PORT").ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);

        let log_level = Self::get_value(map, "LOG_LEVEL")
            .or_else(|| env::var("LOG_LEVEL").ok())
            .unwrap_or_else(|| "info".to_string());

        let features = Self::get_value(map, "FEATURES")
            .map(|s| s.split(',').map(|f| f.trim().to_string()).collect())
            .unwrap_or_default();

        Ok(Config {
            database_url,
            port,
            log_level,
            features,
        })
    }

    fn get_value(map: &HashMap<String, String>, key: &str) -> Option<String> {
        map.get(key).cloned()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }

        if self.port == 0 {
            errors.push("PORT must be greater than 0".to_string());
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!(
                "LOG_LEVEL must be one of: {}",
                valid_log_levels.join(", ")
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_map() {
        let mut map = HashMap::new();
        map.insert("DATABASE_URL".to_string(), "postgres://test".to_string());
        map.insert("PORT".to_string(), "3000".to_string());
        map.insert("LOG_LEVEL".to_string(), "debug".to_string());

        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://test");
        assert_eq!(config.port, 3000);
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_config_defaults() {
        let map = HashMap::new();
        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://localhost:5432/app");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.features.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            port: 0,
            log_level: "invalid".to_string(),
            features: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}