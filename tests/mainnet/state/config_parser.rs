
use std::collections::HashMap;
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
        let var_regex = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                value = value.trim();

                let mut processed_value = value.to_string();
                for cap in var_regex.captures_iter(value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            processed_value = processed_value.replace(&cap[0], &env_value);
                        }
                    }
                }

                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
            server_host=localhost
            server_port=8080
            debug_mode=true
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("server_host").unwrap(), "localhost");
        assert_eq!(config.get("server_port").unwrap(), "8080");
        assert_eq!(config.get("debug_mode").unwrap(), "true");
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let content = r#"
            database_url=postgres://user:${DB_PASSWORD}@localhost/db
            api_key=${NONEXISTENT_VAR:-default_key}
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("database_url").unwrap(), "postgres://user:secret123@localhost/db");
        assert_eq!(config.get_or_default("api_key", "fallback"), "default_key");
    }

    #[test]
    fn test_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "app_name=TestApp").unwrap();
        writeln!(temp_file, "version=1.0.0").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("app_name").unwrap(), "TestApp");
        assert_eq!(config.get("version").unwrap(), "1.0.0");
    }

    #[test]
    fn test_skip_comments_and_blank_lines() {
        let content = r#"
            # This is a comment
            key1=value1
            
            # Another comment
            key2=value2
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.values.len(), 2);
        assert!(config.contains_key("key1"));
        assert!(config.contains_key("key2"));
    }
}use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub enable_https: bool,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&config_str)?;
        
        config.apply_environment_overrides();
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("DB_HOST") {
            self.database.host = host;
        }
        
        if let Ok(port) = env::var("DB_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.database.port = port_num;
            }
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.database.port == 0 {
            return Err("Database port cannot be zero".to_string());
        }
        
        if self.server.port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        
        if self.database.host.is_empty() {
            return Err("Database host cannot be empty".to_string());
        }
        
        if self.database.database_name.is_empty() {
            return Err("Database name cannot be empty".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
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
    
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let config_yaml = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database_name: myapp

server:
  address: 0.0.0.0
  port: 8080
  enable_https: false
  max_connections: 100

log_level: info
cache_ttl: 3600
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_yaml).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("DB_HOST", "remote-host");
        env::set_var("LOG_LEVEL", "debug");
        
        let config_yaml = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database_name: myapp

server:
  address: 0.0.0.0
  port: 8080
  enable_https: false
  max_connections: 100

log_level: info
cache_ttl: 3600
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_yaml).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "remote-host");
        assert_eq!(config.log_level, "debug");
        
        env::remove_var("DB_HOST");
        env::remove_var("LOG_LEVEL");
    }
    
    #[test]
    fn test_validation() {
        let invalid_config_yaml = r#"
database:
  host: ""
  port: 5432
  username: postgres
  password: secret
  database_name: myapp

server:
  address: 0.0.0.0
  port: 0
  enable_https: false
  max_connections: 100

log_level: invalid_level
cache_ttl: 3600
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), invalid_config_yaml).unwrap();
        
        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                self.settings.insert(key, value);
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.settings
            .get(key)
            .cloned()
            .or_else(|| self.defaults.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();

        for key in required_keys {
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
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "timeout=60").unwrap();

        config.load_from_file(temp_file.path()).unwrap();

        assert_eq!(config.get("host"), Some("localhost".to_string()));
        assert_eq!(config.get("port"), Some("8080".to_string()));
        assert_eq!(config.get("timeout"), Some("60".to_string()));
        assert_eq!(config.get("retries"), Some("3".to_string()));
    }

    #[test]
    fn test_validation() {
        let config = Config::new();
        let result = config.validate_required(&["host", "port"]);
        assert!(result.is_err());
        
        let missing = result.unwrap_err();
        assert_eq!(missing, vec!["host", "port"]);
    }

    #[test]
    fn test_get_with_default() {
        let config = Config::new();
        assert_eq!(config.get_with_default("unknown", "default_value"), "default_value");
        assert_eq!(config.get_with_default("timeout", "100"), "30");
    }
}