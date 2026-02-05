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
}