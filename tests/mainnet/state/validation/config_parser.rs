use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 20,
                min_connections: 5,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "logs/app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&config_str)?;
    
    validate_config(&config)?;
    
    Ok(config)
}

pub fn load_config_with_defaults<P: AsRef<Path>>(path: P) -> Result<AppConfig, Box<dyn std::error::Error>> {
    match load_config(path) {
        Ok(config) => Ok(config),
        Err(_) => {
            println!("Using default configuration");
            Ok(AppConfig::default())
        }
    }
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.max_connections < config.database.min_connections {
        return Err("Max connections must be greater than or equal to min connections".to_string());
    }
    
    if config.logging.max_file_size_mb == 0 {
        return Err("Max file size must be greater than 0".to_string());
    }
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(format!("Invalid log level: {}", config.logging.level));
    }
    
    Ok(())
}

pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<(), Box<dyn std::error::Error>> {
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(path, toml_string)?;
    Ok(())
}
use std::collections::HashMap;
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
                let processed_value = Self::interpolate_env_vars(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn interpolate_env_vars(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                
                match env::var(&var_name) {
                    Ok(env_value) => result.push_str(&env_value),
                    Err(_) => result.push_str(&format!("${{{}}}", var_name)),
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
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
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "MAX_CONNECTIONS=100").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "100");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("APP_PORT", "8080");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PORT=${APP_PORT}").unwrap();
        writeln!(file, "HOST=localhost:${APP_PORT}").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("HOST").unwrap(), "localhost:8080");
    }
}