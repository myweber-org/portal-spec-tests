use std::collections::HashMap;
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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                self.values.insert(key, value);
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost".to_string()));
        assert_eq!(config.get("API_KEY"), Some("secret123".to_string()));
        assert_eq!(config.get("NON_EXISTENT"), None);
    }

    #[test]
    fn test_env_override() {
        env::set_var("SPECIAL_KEY", "env_value");
        let config = Config::new();
        assert_eq!(config.get("SPECIAL_KEY"), Some("env_value".to_string()));
    }
}
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.json".to_string());
        
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = serde_json::from_str(&config_content)?;
        
        config.apply_environment_overrides();
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server.host = host;
        }
        
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.server.port = port_num;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
    }
    
    pub fn is_feature_enabled(&self, feature_name: &str) -> bool {
        self.features.get(feature_name).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "server": {
                "host": "localhost",
                "port": 8080,
                "tls_enabled": false
            },
            "database": {
                "url": "postgresql://localhost/app",
                "max_connections": 10,
                "timeout_seconds": 30
            },
            "features": {
                "logging": true,
                "metrics": false
            }
        }"#;
        
        write!(temp_file, "{}", config_json).unwrap();
        
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
        let config = AppConfig::load().unwrap();
        
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert!(config.is_feature_enabled("logging"));
        assert!(!config.is_feature_enabled("metrics"));
        
        env::remove_var("CONFIG_PATH");
    }
    
    #[test]
    fn test_environment_overrides() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "server": {
                "host": "localhost",
                "port": 8080,
                "tls_enabled": false
            },
            "database": {
                "url": "postgresql://localhost/app",
                "max_connections": 10,
                "timeout_seconds": 30
            },
            "features": {}
        }"#;
        
        write!(temp_file, "{}", config_json).unwrap();
        
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        env::set_var("SERVER_HOST", "0.0.0.0");
        env::set_var("DATABASE_URL", "postgresql://prod-db/app");
        
        let config = AppConfig::load().unwrap();
        
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.database.url, "postgresql://prod-db/app");
        
        env::remove_var("CONFIG_PATH");
        env::remove_var("SERVER_HOST");
        env::remove_var("DATABASE_URL");
    }
}use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
    pub custom_settings: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
        
        let config: toml::Value = config_content.parse()
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        let database_url = Self::get_string(&config, "database.url")
            .or_else(|| env::var("DATABASE_URL").ok())
            .ok_or("Database URL not found in config or environment")?;
        
        let api_key = Self::get_string(&config, "api.key")
            .or_else(|| env::var("API_KEY").ok())
            .ok_or("API key not found in config or environment")?;
        
        let debug_mode = Self::get_bool(&config, "app.debug")
            .unwrap_or_else(|| env::var("DEBUG_MODE").unwrap_or_default() == "true");
        
        let port = Self::get_u16(&config, "app.port")
            .or_else(|| env::var("PORT").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(8080);
        
        let mut custom_settings = HashMap::new();
        if let Some(table) = config.get("custom") {
            if let Some(custom_table) = table.as_table() {
                for (key, value) in custom_table {
                    if let Some(str_val) = value.as_str() {
                        custom_settings.insert(key.clone(), str_val.to_string());
                    }
                }
            }
        }
        
        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
            custom_settings,
        })
    }
    
    fn get_string(config: &toml::Value, path: &str) -> Option<String> {
        let mut current = config;
        for part in path.split('.') {
            current = current.get(part)?;
        }
        current.as_str().map(|s| s.to_string())
    }
    
    fn get_bool(config: &toml::Value, path: &str) -> Option<bool> {
        let mut current = config;
        for part in path.split('.') {
            current = current.get(part)?;
        }
        current.as_bool()
    }
    
    fn get_u16(config: &toml::Value, path: &str) -> Option<u16> {
        let mut current = config;
        for part in path.split('.') {
            current = current.get(part)?;
        }
        current.as_integer().and_then(|n| {
            if n >= 0 && n <= u16::MAX as i64 {
                Some(n as u16)
            } else {
                None
            }
        })
    }
}