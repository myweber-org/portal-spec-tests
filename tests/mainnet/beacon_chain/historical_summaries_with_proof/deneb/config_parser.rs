
use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, PartialEq)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let parsed: toml::Value = content.parse()
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;
        
        let database_table = parsed.get("database")
            .ok_or("Missing 'database' section")?
            .as_table()
            .ok_or("'database' section must be a table")?;
        
        let server_table = parsed.get("server")
            .ok_or("Missing 'server' section")?
            .as_table()
            .ok_or("'server' section must be a table")?;
        
        let features_table = parsed.get("features")
            .map(|v| v.as_table().unwrap_or(&toml::value::Table::new()))
            .unwrap_or(&toml::value::Table::new());
        
        let database = DatabaseConfig {
            host: database_table.get("host")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid database.host")?
                .to_string(),
            port: database_table.get("port")
                .and_then(|v| v.as_integer())
                .ok_or("Missing or invalid database.port")?
                .try_into()
                .map_err(|_| "database.port out of range")?,
            username: database_table.get("username")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid database.username")?
                .to_string(),
            password: database_table.get("password")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid database.password")?
                .to_string(),
            name: database_table.get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid database.name")?
                .to_string(),
        };
        
        let server = ServerConfig {
            host: server_table.get("host")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid server.host")?
                .to_string(),
            port: server_table.get("port")
                .and_then(|v| v.as_integer())
                .ok_or("Missing or invalid server.port")?
                .try_into()
                .map_err(|_| "server.port out of range")?,
            tls_enabled: server_table.get("tls_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        };
        
        let mut features = HashMap::new();
        for (key, value) in features_table {
            if let Some(bool_val) = value.as_bool() {
                features.insert(key.clone(), bool_val);
            }
        }
        
        Ok(Config {
            database,
            server,
            features,
        })
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.database.port == 0 {
            return Err("Database port cannot be 0".to_string());
        }
        
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        
        if self.server.tls_enabled && self.server.port == 80 {
            return Err("TLS should not be enabled on port 80".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            name = "app_db"
            
            [server]
            host = "0.0.0.0"
            port = 8080
            tls_enabled = true
            
            [features]
            caching = true
            logging = false
        "#;
        
        write!(temp_file, "{}", config_content).unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.tls_enabled, true);
        assert_eq!(config.features.get("caching"), Some(&true));
        assert_eq!(config.features.get("logging"), Some(&false));
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_missing_section() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            [database]
            host = "localhost"
            port = 5432
        "#;
        
        write!(temp_file, "{}", config_content).unwrap();
        
        let result = Config::from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'server' section"));
    }
    
    #[test]
    fn test_validation_failure() {
        let config = Config {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 0,
                username: "admin".to_string(),
                password: "secret".to_string(),
                name: "db".to_string(),
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 80,
                tls_enabled: true,
            },
            features: HashMap::new(),
        };
        
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Database port cannot be 0") || 
                err.contains("TLS should not be enabled on port 80"));
    }
}use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_address: String,
    pub server_port: u16,
    pub max_connections: usize,
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
        let config_str = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(_) => {
                eprintln!("Failed to load config file, using default values");
                AppConfig::default()
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
            return Err(format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }
        Ok(())
    }

    pub fn server_endpoint(&self) -> String {
        format!("{}:{}", self.server_address, self.server_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server_address, "127.0.0.1");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.enable_logging, true);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_server_endpoint() {
        let config = AppConfig {
            server_address: String::from("localhost"),
            server_port: 3000,
            ..Default::default()
        };
        assert_eq!(config.server_endpoint(), "localhost:3000");
    }

    #[test]
    fn test_validation() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.max_connections = 0;
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.log_level = String::from("invalid");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_load_from_file() {
        let toml_content = r#"
            server_address = "0.0.0.0"
            server_port = 9000
            max_connections = 500
            enable_logging = false
            log_level = "debug"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_address, "0.0.0.0");
        assert_eq!(config.server_port, 9000);
        assert_eq!(config.max_connections, 500);
        assert_eq!(config.enable_logging, false);
        assert_eq!(config.log_level, "debug");
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut values = HashMap::new();
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
            let raw_value = parts[1].trim().to_string();
            let value = Self::resolve_env_vars(&raw_value);

            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn resolve_env_vars(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                
                match env::var(&var_name) {
                    Ok(val) => result.push_str(&val),
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
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
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
        writeln!(file, "HOST=localhost\nPORT=8080").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "postgres");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://${DB_HOST}:5432/db").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some(&"postgres://postgres:5432/db".to_string()));
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
    pub tls_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub rotation: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                tls_enabled: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                rotation: "daily".to_string(),
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

    pub fn from_file_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database.max_connections == 0 {
            return Err("Database max connections cannot be zero".to_string());
        }
        if !["trace", "debug", "info", "warn", "error"].contains(&self.logging.level.as_str()) {
            return Err(format!("Invalid log level: {}", self.logging.level));
        }
        Ok(())
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_from_file() {
        let toml_content = r#"
            [server]
            host = "0.0.0.0"
            port = 9000
            tls_enabled = true

            [database]
            url = "postgresql://prod:5432/appdb"
            max_connections = 50
            timeout_seconds = 60

            [logging]
            level = "debug"
            file_path = "/var/log/app.log"
            rotation = "hourly"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.database.max_connections, 50);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.file_path, Some("/var/log/app.log".to_string()));
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
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
                let processed_value = Self::interpolate_env(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn interpolate_env(value: &str) -> String {
        let mut result = value.to_string();
        for (key, env_value) in env::vars() {
            let placeholder = format!("${{{}}}", key);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &env_value);
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

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn merge(&mut self, other: Config) {
        for (key, value) in other.values {
            self.values.insert(key, value);
        }
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
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "MAX_CONNECTIONS=100").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "100");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(!config.contains_key("NONEXISTENT"));
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("APP_PORT", "8080");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PORT=${{APP_PORT}}").unwrap();
        writeln!(file, "HOST=localhost:${{APP_PORT}}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("HOST").unwrap(), "localhost:8080");
    }

    #[test]
    fn test_merge_configs() {
        let mut config1 = Config {
            values: HashMap::from([
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ]),
        };

        let config2 = Config {
            values: HashMap::from([
                ("key2".to_string(), "overridden".to_string()),
                ("key3".to_string(), "value3".to_string()),
            ]),
        };

        config1.merge(config2);
        assert_eq!(config1.get("key1").unwrap(), "value1");
        assert_eq!(config1.get("key2").unwrap(), "overridden");
        assert_eq!(config1.get("key3").unwrap(), "value3");
    }
}