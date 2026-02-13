
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
}