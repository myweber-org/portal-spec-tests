use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_address: String,
    pub server_port: u16,
    pub max_connections: u32,
    pub enable_logging: bool,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_address: String::from("127.0.0.1"),
            server_port: 8080,
            max_connections: 100,
            enable_logging: true,
            log_level: String::from("info"),
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be zero"));
        }
        if self.max_connections == 0 {
            return Err(String::from("Max connections must be greater than zero"));
        }
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        Ok(())
    }

    pub fn server_endpoint(&self) -> String {
        format!("{}:{}", self.server_address, self.server_port)
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
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("API_KEY", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "KEY=$API_KEY").unwrap();
        writeln!(file, "NORMAL=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("KEY").unwrap(), "secret123");
        assert_eq!(config.get("NORMAL").unwrap(), "value");
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub cache_size: usize,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config_map.insert(key, value);
            }
        }

        Self::from_map(&config_map)
    }

    pub fn from_env() -> Result<Self, String> {
        let mut config_map = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config_map.insert(config_key, value);
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = map
            .get("database_url")
            .map(|s| s.to_string())
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/mydb".to_string());

        let port = map
            .get("port")
            .and_then(|s| s.parse().ok())
            .or_else(|| env::var("PORT").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(8080);

        let log_level = map
            .get("log_level")
            .map(|s| s.to_string())
            .or_else(|| env::var("LOG_LEVEL").ok())
            .unwrap_or_else(|| "info".to_string());

        let cache_size = map
            .get("cache_size")
            .and_then(|s| s.parse().ok())
            .or_else(|| env::var("CACHE_SIZE").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(1000);

        Ok(Config {
            database_url,
            port,
            log_level,
            cache_size,
        })
    }

    pub fn merge(self, other: Self) -> Self {
        Config {
            database_url: other.database_url,
            port: other.port,
            log_level: other.log_level,
            cache_size: other.cache_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "database_url=postgres://test:5432/db").unwrap();
        writeln!(file, "port=3000").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "log_level=debug").unwrap();
        writeln!(file, "cache_size=500").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://test:5432/db");
        assert_eq!(config.port, 3000);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_size, 500);
    }

    #[test]
    fn test_from_env() {
        env::set_var("APP_DATABASE_URL", "postgres://env:5432/db");
        env::set_var("APP_PORT", "4000");
        env::set_var("APP_LOG_LEVEL", "trace");
        env::set_var("APP_CACHE_SIZE", "200");

        let config = Config::from_env().unwrap();
        assert_eq!(config.database_url, "postgres://env:5432/db");
        assert_eq!(config.port, 4000);
        assert_eq!(config.log_level, "trace");
        assert_eq!(config.cache_size, 200);

        env::remove_var("APP_DATABASE_URL");
        env::remove_var("APP_PORT");
        env::remove_var("APP_LOG_LEVEL");
        env::remove_var("APP_CACHE_SIZE");
    }

    #[test]
    fn test_merge() {
        let config1 = Config {
            database_url: "url1".to_string(),
            port: 1000,
            log_level: "info".to_string(),
            cache_size: 100,
        };

        let config2 = Config {
            database_url: "url2".to_string(),
            port: 2000,
            log_level: "debug".to_string(),
            cache_size: 200,
        };

        let merged = config1.merge(config2);
        assert_eq!(merged.database_url, "url2");
        assert_eq!(merged.port, 2000);
        assert_eq!(merged.log_level, "debug");
        assert_eq!(merged.cache_size, 200);
    }
}