use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.*?)\s*$").unwrap();
        let var_re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();

                // Replace environment variables
                for var_caps in var_re.captures_iter(&value) {
                    let var_name = &var_caps[1];
                    if let Ok(var_value) = env::var(var_name) {
                        value = value.replace(&var_caps[0], &var_value);
                    }
                }

                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }
        }

        Ok(())
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

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = r#"
            database_host = localhost
            database_port = 5432
            # This is a comment
            api_key = secret_value
        "#;

        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("database_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("database_port"), Some(&"5432".to_string()));
        assert_eq!(parser.get("api_key"), Some(&"secret_value".to_string()));
        assert_eq!(parser.get("nonexistent"), None);
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("APP_MODE", "production");
        
        let mut parser = ConfigParser::new();
        let config = r#"
            mode = ${APP_MODE}
            path = /home/${USER}/data
        "#;

        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("mode"), Some(&"production".to_string()));
    }

    #[test]
    fn test_invalid_syntax() {
        let mut parser = ConfigParser::new();
        let config = r#"
            valid_key = valid_value
            invalid line without equals
            another_valid = value
        "#;

        assert!(parser.load_from_str(config).is_err());
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&content)?;
        config.resolve_environment_variables();
        Ok(config)
    }

    fn resolve_environment_variables(&mut self) {
        self.database.host = Self::get_env_or_default(
            "DB_HOST",
            &self.database.host,
        );
        self.database.port = Self::get_env_or_default(
            "DB_PORT",
            &self.database.port.to_string(),
        ).parse().unwrap_or(self.database.port);
        self.database.username = Self::get_env_or_default(
            "DB_USER",
            &self.database.username,
        );
        self.database.password = Self::get_env_or_default(
            "DB_PASS",
            &self.database.password,
        );
        self.database.database_name = Self::get_env_or_default(
            "DB_NAME",
            &self.database.database_name,
        );

        self.server.host = Self::get_env_or_default(
            "SERVER_HOST",
            &self.server.host,
        );
        self.server.port = Self::get_env_or_default(
            "SERVER_PORT",
            &self.server.port.to_string(),
        ).parse().unwrap_or(self.server.port);
        self.server.max_connections = Self::get_env_or_default(
            "MAX_CONNECTIONS",
            &self.server.max_connections.to_string(),
        ).parse().unwrap_or(self.server.max_connections);
        self.server.timeout_seconds = Self::get_env_or_default(
            "TIMEOUT_SECONDS",
            &self.server.timeout_seconds.to_string(),
        ).parse().unwrap_or(self.server.timeout_seconds);
    }

    fn get_env_or_default(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database.host.is_empty() {
            errors.push("Database host cannot be empty".to_string());
        }
        if self.database.port == 0 {
            errors.push("Database port must be greater than 0".to_string());
        }
        if self.database.username.is_empty() {
            errors.push("Database username cannot be empty".to_string());
        }
        if self.server.port == 0 {
            errors.push("Server port must be greater than 0".to_string());
        }
        if self.server.max_connections == 0 {
            errors.push("Max connections must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let yaml_content = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database_name: mydb
server:
  host: 0.0.0.0
  port: 8080
  max_connections: 100
  timeout_seconds: 30
features:
  caching: true
  logging: false
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), yaml_content).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.features.get("caching"), Some(&true));
    }

    #[test]
    fn test_environment_variable_override() {
        env::set_var("DB_HOST", "prod-db.example.com");
        env::set_var("SERVER_PORT", "9090");

        let yaml_content = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database_name: mydb
server:
  host: 0.0.0.0
  port: 8080
  max_connections: 100
  timeout_seconds: 30
features: {}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), yaml_content).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database.host, "prod-db.example.com");
        assert_eq!(config.server.port, 9090);

        env::remove_var("DB_HOST");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database: DatabaseConfig {
                host: "".to_string(),
                port: 0,
                username: "".to_string(),
                password: "secret".to_string(),
                database_name: "mydb".to_string(),
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 0,
                max_connections: 0,
                timeout_seconds: 30,
            },
            features: HashMap::new(),
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 4);
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub feature_flags: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config_map = HashMap::new();
        
        for line in content.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let raw_value = parts[1].trim().to_string();
                let value = Self::interpolate_env_vars(&raw_value);
                config_map.insert(key, value);
            }
        }
        
        Ok(Config {
            database_url: config_map
                .get("DATABASE_URL")
                .cloned()
                .unwrap_or_else(|| "postgresql://localhost:5432/app".to_string()),
            server_port: config_map
                .get("SERVER_PORT")
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            log_level: config_map
                .get("LOG_LEVEL")
                .cloned()
                .unwrap_or_else(|| "info".to_string()),
            feature_flags: config_map
                .iter()
                .filter(|(k, _)| k.starts_with("FEATURE_"))
                .map(|(k, v)| (k.clone(), v == "true" || v == "1"))
                .collect(),
        })
    }
    
    fn interpolate_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        
        if value.starts_with("${") && value.ends_with('}') {
            let env_key = &value[2..value.len() - 1];
            if let Ok(env_value) = env::var(env_key) {
                result = env_value;
            }
        }
        
        result
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }
        
        if self.server_port == 0 {
            errors.push("SERVER_PORT must be greater than 0".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!(
                "LOG_LEVEL must be one of: {}",
                valid_log_levels.join(", ")
            ));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
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
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "DATABASE_URL=postgresql://localhost:5432/test").unwrap();
        writeln!(config_file, "SERVER_PORT=3000").unwrap();
        writeln!(config_file, "LOG_LEVEL=debug").unwrap();
        writeln!(config_file, "FEATURE_AUTH=true").unwrap();
        writeln!(config_file, "FEATURE_CACHE=1").unwrap();
        
        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.database_url, "postgresql://localhost:5432/test");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.feature_flags.get("FEATURE_AUTH"), Some(&true));
        assert_eq!(config.feature_flags.get("FEATURE_CACHE"), Some(&true));
    }
    
    #[test]
    fn test_env_var_interpolation() {
        env::set_var("DB_HOST", "localhost");
        
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "DATABASE_URL=${{DB_HOST}}").unwrap();
        
        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "localhost");
    }
    
    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            server_port: 0,
            log_level: "invalid".to_string(),
            feature_flags: HashMap::new(),
        };
        
        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        if let Err(errors) = validation_result {
            assert!(errors.len() >= 2);
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
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let raw_value = parts[1].trim().to_string();
                let value = Self::expand_env_vars(&raw_value);
                values.insert(key, value);
            }
        }
        
        Ok(Config { values })
    }
    
    fn expand_env_vars(input: &str) -> String {
        let mut result = input.to_string();
        
        for (key, value) in env::vars() {
            let placeholder = format!("${}", key);
            result = result.replace(&placeholder, &value);
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
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert!(config.get("NONEXISTENT").is_none());
    }
    
    #[test]
    fn test_env_var_expansion() {
        env::set_var("APP_ENV", "production");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "ENVIRONMENT=${APP_ENV}").unwrap();
        writeln!(file, "HOST=api.${APP_ENV}.example.com").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("ENVIRONMENT").unwrap(), "production");
        assert_eq!(config.get("HOST").unwrap(), "api.production.example.com");
    }
}