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
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
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
        writeln!(file, "APP_NAME=myapp").unwrap();
        writeln!(file, "VERSION=1.0.0").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "  SPACED_KEY  =  spaced_value  ").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"myapp".to_string()));
        assert_eq!(config.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(config.get("SPACED_KEY"), Some(&"spaced_value".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("DB_HOST", "localhost");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE=${DB_HOST}").unwrap();
        writeln!(file, "STATIC_VALUE=production").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE"), Some(&"localhost".to_string()));
        assert_eq!(config.get("STATIC_VALUE"), Some(&"production".to_string()));
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
                let processed_value = Self::interpolate_env_vars(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn interpolate_env_vars(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next(); // Skip '}'
                        break;
                    }
                    var_name.push(ch);
                    chars.next();
                }
                
                if !var_name.is_empty() {
                    result.push_str(&env::var(&var_name).unwrap_or_default());
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
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "APP_NAME=MyApp").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "VERSION=1.0.0").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "DEBUG=true").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"MyApp".to_string()));
        assert_eq!(config.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(config.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("HOME_DIR", "/home/user");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PATH=${HOME_DIR}/data").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PATH"), Some(&"/home/user/data".to_string()));
    }
}use serde::{Deserialize, Serialize};
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
    pub file_path: Option<String>,
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
                file_path: None,
                max_file_size_mb: 100,
            },
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".into());
        }
        
        if self.database.max_connections < self.database.min_connections {
            return Err("Max connections must be greater than or equal to min connections".into());
        }

        if self.database.max_connections == 0 {
            return Err("Max connections cannot be zero".into());
        }

        if let Some(ref path) = self.logging.file_path {
            if path.is_empty() {
                return Err("Log file path cannot be empty if specified".into());
            }
        }

        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(format!("Invalid log level: {}", self.logging.level).into());
        }

        Ok(())
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn merge_with_defaults(mut self) -> Self {
        let default = AppConfig::default();
        
        if self.server.host.is_empty() {
            self.server.host = default.server.host;
        }
        
        if self.server.port == 0 {
            self.server.port = default.server.port;
        }
        
        if self.server.timeout_seconds == 0 {
            self.server.timeout_seconds = default.server.timeout_seconds;
        }
        
        if self.database.url.is_empty() {
            self.database.url = default.database.url;
        }
        
        if self.database.max_connections == 0 {
            self.database.max_connections = default.database.max_connections;
        }
        
        if self.database.min_connections == 0 {
            self.database.min_connections = default.database.min_connections;
        }
        
        if self.logging.level.is_empty() {
            self.logging.level = default.logging.level;
        }
        
        if self.logging.max_file_size_mb == 0 {
            self.logging.max_file_size_mb = default.logging.max_file_size_mb;
        }

        self
    }
}