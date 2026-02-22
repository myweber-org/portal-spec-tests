
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

    fn process_value(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();

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
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
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
        self.values.get(key).cloned().unwrap_or(default.to_string())
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
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "DEBUG=true").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"myapp".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://${DB_HOST}:5432/mydb").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(
            config.get("DATABASE_URL"),
            Some(&"postgres://localhost:5432/mydb".to_string())
        );
    }

    #[test]
    fn test_missing_env_var() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "URL=https://${UNKNOWN_VAR}/api").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(
            config.get("URL"),
            Some(&"https://${UNKNOWN_VAR}/api".to_string())
        );
    }
}
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug_mode: bool,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        
        if self.server.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }
        
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        Ok(())
    }
    
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database_name
        )
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
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
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
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${DB_PASSWORD}").unwrap();
        writeln!(file, "NORMAL=plain_value").unwrap();
        writeln!(file, "MISSING=${UNDEFINED_VAR}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"plain_value".to_string()));
        assert_eq!(config.get("MISSING"), Some(&"${UNDEFINED_VAR}".to_string()));
    }
}use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size: u64,
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
                connection_timeout: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "app.log".to_string(),
                max_file_size: 10485760,
            },
        }
    }
}

pub struct ConfigParser;

impl ConfigParser {
    pub fn from_file(path: &str) -> Result<AppConfig, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        Self::validate_config(&mut config)?;
        Ok(config)
    }
    
    pub fn from_env() -> Result<AppConfig, String> {
        let mut config = AppConfig::default();
        
        if let Ok(host) = std::env::var("SERVER_HOST") {
            config.server.host = host;
        }
        
        if let Ok(port) = std::env::var("SERVER_PORT") {
            config.server.port = port.parse()
                .map_err(|_| "Invalid SERVER_PORT value".to_string())?;
        }
        
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database.url = db_url;
        }
        
        Self::validate_config(&mut config)?;
        Ok(config)
    }
    
    pub fn merge_with_default(mut config: AppConfig) -> AppConfig {
        let default = AppConfig::default();
        
        if config.server.host.is_empty() {
            config.server.host = default.server.host;
        }
        
        if config.server.port == 0 {
            config.server.port = default.server.port;
        }
        
        if config.server.timeout_seconds == 0 {
            config.server.timeout_seconds = default.server.timeout_seconds;
        }
        
        if config.database.url.is_empty() {
            config.database.url = default.database.url;
        }
        
        if config.database.max_connections == 0 {
            config.database.max_connections = default.database.max_connections;
        }
        
        if config.database.min_connections == 0 {
            config.database.min_connections = default.database.min_connections;
        }
        
        if config.database.connection_timeout == 0 {
            config.database.connection_timeout = default.database.connection_timeout;
        }
        
        if config.logging.level.is_empty() {
            config.logging.level = default.logging.level;
        }
        
        if config.logging.file_path.is_empty() {
            config.logging.file_path = default.logging.file_path;
        }
        
        if config.logging.max_file_size == 0 {
            config.logging.max_file_size = default.logging.max_file_size;
        }
        
        config
    }
    
    fn validate_config(config: &AppConfig) -> Result<(), String> {
        if config.server.port > 65535 {
            return Err("Server port must be between 1 and 65535".to_string());
        }
        
        if config.database.max_connections < config.database.min_connections {
            return Err("Max connections must be greater than or equal to min connections".to_string());
        }
        
        if config.database.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&config.logging.level.as_str()) {
            return Err(format!("Invalid log level. Must be one of: {:?}", valid_log_levels));
        }
        
        Ok(())
    }
    
    pub fn to_toml(&self, config: &AppConfig) -> Result<String, String> {
        toml::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config to TOML: {}", e))
    }
    
    pub fn to_env_vars(config: &AppConfig) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        
        env_vars.insert("SERVER_HOST".to_string(), config.server.host.clone());
        env_vars.insert("SERVER_PORT".to_string(), config.server.port.to_string());
        env_vars.insert("SERVER_TIMEOUT".to_string(), config.server.timeout_seconds.to_string());
        
        env_vars.insert("DATABASE_URL".to_string(), config.database.url.clone());
        env_vars.insert("DATABASE_MAX_CONNECTIONS".to_string(), config.database.max_connections.to_string());
        env_vars.insert("DATABASE_MIN_CONNECTIONS".to_string(), config.database.min_connections.to_string());
        env_vars.insert("DATABASE_CONNECTION_TIMEOUT".to_string(), config.database.connection_timeout.to_string());
        
        env_vars.insert("LOG_LEVEL".to_string(), config.logging.level.clone());
        env_vars.insert("LOG_FILE_PATH".to_string(), config.logging.file_path.clone());
        env_vars.insert("LOG_MAX_FILE_SIZE".to_string(), config.logging.max_file_size.to_string());
        
        env_vars
    }
}