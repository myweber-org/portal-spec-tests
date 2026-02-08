use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let processed_value = Self::process_value(value.trim());
                settings.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { settings })
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
        self.settings.get(key)
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
        writeln!(file, "PASSWORD=$DB_PASSWORD").unwrap();
        writeln!(file, "NORMAL=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"value".to_string()));
    }
}use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut values = HashMap::new();
        let var_regex = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = line.split_once('=') {
                let key = key.trim().to_string();
                value = value.trim();

                for cap in var_regex.captures_iter(value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            value = value.replace(&cap[0], &env_value);
                        }
                    }
                }

                values.insert(key, value.to_string());
            }
        }

        Ok(Config { values })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
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
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}, using defaults", e);
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
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut config = Config::new();

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                config.values.insert(key, value);
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn merge_env(&mut self) {
        for (key, value) in env::vars() {
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
    fn test_config_creation() {
        let config = Config::new();
        assert!(config.get("NON_EXISTENT").is_none());
    }

    #[test]
    fn test_config_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("API_KEY").unwrap(), "secret123");
    }

    #[test]
    fn test_env_override() {
        env::set_var("TEST_KEY", "env_value");
        
        let mut config = Config::new();
        config.set("TEST_KEY", "file_value");
        
        assert_eq!(config.get("TEST_KEY").unwrap(), "env_value");
        
        env::remove_var("TEST_KEY");
    }

    #[test]
    fn test_default_value() {
        let config = Config::new();
        assert_eq!(config.get_with_default("MISSING", "default_value"), "default_value");
    }
}use std::collections::HashMap;
use std::fs;
use toml::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub enable_compression: bool,
}

#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub enable_logging: bool,
    pub enable_metrics: bool,
    pub cache_enabled: bool,
    pub cache_ttl: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let parsed: Value = content.parse()
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;
        
        Self::from_toml_value(&parsed)
    }
    
    fn from_toml_value(value: &Value) -> Result<Self, String> {
        let database = Self::parse_database_config(value)?;
        let server = Self::parse_server_config(value)?;
        let features = Self::parse_feature_flags(value)?;
        
        Ok(Config {
            database,
            server,
            features,
        })
    }
    
    fn parse_database_config(value: &Value) -> Result<DatabaseConfig, String> {
        let db_table = value.get("database")
            .ok_or("Missing 'database' section")?
            .as_table()
            .ok_or("'database' section must be a table")?;
        
        Ok(DatabaseConfig {
            host: Self::get_string(db_table, "host")?,
            port: Self::get_u16(db_table, "port")?,
            username: Self::get_string(db_table, "username")?,
            password: Self::get_string(db_table, "password")?,
            database_name: Self::get_string(db_table, "database_name")?,
            pool_size: Self::get_u32(db_table, "pool_size")?,
        })
    }
    
    fn parse_server_config(value: &Value) -> Result<ServerConfig, String> {
        let server_table = value.get("server")
            .ok_or("Missing 'server' section")?
            .as_table()
            .ok_or("'server' section must be a table")?;
        
        Ok(ServerConfig {
            host: Self::get_string(server_table, "host")?,
            port: Self::get_u16(server_table, "port")?,
            max_connections: Self::get_u32(server_table, "max_connections")?,
            timeout_seconds: Self::get_u64(server_table, "timeout_seconds")?,
            enable_compression: Self::get_bool(server_table, "enable_compression")?,
        })
    }
    
    fn parse_feature_flags(value: &Value) -> Result<FeatureFlags, String> {
        let features_table = value.get("features")
            .ok_or("Missing 'features' section")?
            .as_table()
            .ok_or("'features' section must be a table")?;
        
        Ok(FeatureFlags {
            enable_logging: Self::get_bool(features_table, "enable_logging")?,
            enable_metrics: Self::get_bool(features_table, "enable_metrics")?,
            cache_enabled: Self::get_bool(features_table, "cache_enabled")?,
            cache_ttl: Self::get_u64(features_table, "cache_ttl")?,
        })
    }
    
    fn get_string(table: &toml::map::Map<String, Value>, key: &str) -> Result<String, String> {
        table.get(key)
            .ok_or(format!("Missing '{}'", key))?
            .as_str()
            .map(|s| s.to_string())
            .ok_or(format!("'{}' must be a string", key))
    }
    
    fn get_u16(table: &toml::map::Map<String, Value>, key: &str) -> Result<u16, String> {
        table.get(key)
            .ok_or(format!("Missing '{}'", key))?
            .as_integer()
            .and_then(|i| i.try_into().ok())
            .ok_or(format!("'{}' must be a valid u16", key))
    }
    
    fn get_u32(table: &toml::map::Map<String, Value>, key: &str) -> Result<u32, String> {
        table.get(key)
            .ok_or(format!("Missing '{}'", key))?
            .as_integer()
            .and_then(|i| i.try_into().ok())
            .ok_or(format!("'{}' must be a valid u32", key))
    }
    
    fn get_u64(table: &toml::map::Map<String, Value>, key: &str) -> Result<u64, String> {
        table.get(key)
            .ok_or(format!("Missing '{}'", key))?
            .as_integer()
            .and_then(|i| i.try_into().ok())
            .ok_or(format!("'{}' must be a valid u64", key))
    }
    
    fn get_bool(table: &toml::map::Map<String, Value>, key: &str) -> Result<bool, String> {
        table.get(key)
            .ok_or(format!("Missing '{}'", key))?
            .as_bool()
            .ok_or(format!("'{}' must be a boolean", key))
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database.port == 0 {
            errors.push("Database port cannot be 0".to_string());
        }
        
        if self.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }
        
        if self.database.pool_size == 0 {
            errors.push("Database pool size must be greater than 0".to_string());
        }
        
        if self.server.max_connections == 0 {
            errors.push("Server max connections must be greater than 0".to_string());
        }
        
        if self.features.cache_enabled && self.features.cache_ttl == 0 {
            errors.push("Cache TTL must be greater than 0 when cache is enabled".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        map.insert("database.host".to_string(), self.database.host.clone());
        map.insert("database.port".to_string(), self.database.port.to_string());
        map.insert("database.username".to_string(), self.database.username.clone());
        map.insert("database.database_name".to_string(), self.database.database_name.clone());
        map.insert("database.pool_size".to_string(), self.database.pool_size.to_string());
        
        map.insert("server.host".to_string(), self.server.host.clone());
        map.insert("server.port".to_string(), self.server.port.to_string());
        map.insert("server.max_connections".to_string(), self.server.max_connections.to_string());
        map.insert("server.timeout_seconds".to_string(), self.server.timeout_seconds.to_string());
        map.insert("server.enable_compression".to_string(), self.server.enable_compression.to_string());
        
        map.insert("features.enable_logging".to_string(), self.features.enable_logging.to_string());
        map.insert("features.enable_metrics".to_string(), self.features.enable_metrics.to_string());
        map.insert("features.cache_enabled".to_string(), self.features.cache_enabled.to_string());
        map.insert("features.cache_ttl".to_string(), self.features.cache_ttl.to_string());
        
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            database_name = "app_db"
            pool_size = 10
            
            [server]
            host = "0.0.0.0"
            port = 8080
            max_connections = 100
            timeout_seconds = 30
            enable_compression = true
            
            [features]
            enable_logging = true
            enable_metrics = false
            cache_enabled = true
            cache_ttl = 300
        "#;
        
        let parsed: Value = toml_content.parse().unwrap();
        let config = Config::from_toml_value(&parsed).unwrap();
        
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.features.cache_ttl, 300);
        
        let validation_result = config.validate();
        assert!(validation_result.is_ok());
    }
    
    #[test]
    fn test_config_validation_failure() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 0
            username = "admin"
            password = "secret"
            database_name = "app_db"
            pool_size = 0
            
            [server]
            host = "0.0.0.0"
            port = 8080
            max_connections = 0
            timeout_seconds = 30
            enable_compression = true
            
            [features]
            enable_logging = true
            enable_metrics = false
            cache_enabled = true
            cache_ttl = 0
        "#;
        
        let parsed: Value = toml_content.parse().unwrap();
        let config = Config::from_toml_value(&parsed).unwrap();
        
        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        if let Err(errors) = validation_result {
            assert!(errors.len() >= 4);
        }
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
    fn test_env_interpolation() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=$DB_PASSWORD").unwrap();
        writeln!(file, "NORMAL=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"value".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=found").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "found");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}
use std::collections::HashMap;
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
                return Err(format!("Invalid config line: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::interpolate_env_vars(&raw_value);

            values.insert(key, value);
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
                        if let Ok(var_value) = env::var(&var_name) {
                            result.push_str(&var_value);
                        }
                        break;
                    } else {
                        var_name.push(ch);
                        chars.next();
                    }
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
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("APP_SECRET", "mysecret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET=${{APP_SECRET}}").unwrap();
        writeln!(file, "PATH=/home/${{USER}}/data").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET"), Some(&"mysecret123".to_string()));
        
        if let Ok(user) = env::var("USER") {
            let expected = format!("/home/{}/data", user);
            assert_eq!(config.get("PATH"), Some(&expected));
        }
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}