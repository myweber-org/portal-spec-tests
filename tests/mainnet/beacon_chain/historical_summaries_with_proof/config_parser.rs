use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub defaults: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            defaults: HashMap::from([
                ("timeout".to_string(), "30".to_string()),
                ("retries".to_string(), "3".to_string()),
                ("log_level".to_string(), "info".to_string()),
            ]),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if value.is_empty() {
                return Err(format!("Empty value for key: {}", key));
            }

            self.settings.insert(key, value);
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }

    pub fn validate_required(&self, keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();

        for key in keys {
            if !self.settings.contains_key(*key) && !self.defaults.contains_key(*key) {
                missing.push(key.to_string());
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    pub fn merge(&mut self, other: Config) {
        for (key, value) in other.settings {
            self.settings.insert(key, value);
        }
        for (key, value) in other.defaults {
            self.defaults.entry(key).or_insert(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "timeout=60").unwrap();

        config.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"60".to_string()));
        assert_eq!(config.get("retries"), Some(&"3".to_string()));
    }

    #[test]
    fn test_validation() {
        let config = Config::new();
        let required = vec!["timeout", "retries", "missing_key"];
        
        match config.validate_required(&required) {
            Ok(_) => panic!("Should have failed validation"),
            Err(missing) => {
                assert_eq!(missing.len(), 1);
                assert_eq!(missing[0], "missing_key");
            }
        }
    }

    #[test]
    fn test_get_with_default() {
        let config = Config::new();
        assert_eq!(config.get_with_default("timeout", "100"), "30");
        assert_eq!(config.get_with_default("nonexistent", "default_value"), "default_value");
    }
}use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub enable_tls: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 100,
            enable_tls: false,
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: ServerConfig = toml::from_str(&config_str)?;
    
    validate_config(&config)?;
    Ok(config)
}

pub fn load_config_with_defaults<P: AsRef<Path>>(path: P) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    match load_config(path) {
        Ok(config) => Ok(config),
        Err(_) => {
            println!("Using default configuration");
            Ok(ServerConfig::default())
        }
    }
}

fn validate_config(config: &ServerConfig) -> Result<(), String> {
    if config.port == 0 {
        return Err("Port cannot be 0".to_string());
    }
    
    if config.max_connections > 10000 {
        return Err("Maximum connections cannot exceed 10000".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert!(!config.enable_tls);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = ServerConfig {
            host: "localhost".to_string(),
            port: 3000,
            max_connections: 500,
            enable_tls: true,
        };
        
        assert!(validate_config(&valid_config).is_ok());

        let invalid_port_config = ServerConfig {
            host: "localhost".to_string(),
            port: 0,
            max_connections: 500,
            enable_tls: true,
        };
        
        assert!(validate_config(&invalid_port_config).is_err());
    }

    #[test]
    fn test_load_config_from_file() {
        let config_content = r#"
            host = "0.0.0.0"
            port = 9000
            max_connections = 2000
            enable_tls = true
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), config_content).unwrap();

        let config = load_config(temp_file.path()).unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9000);
        assert_eq!(config.max_connections, 2000);
        assert!(config.enable_tls);
    }
}