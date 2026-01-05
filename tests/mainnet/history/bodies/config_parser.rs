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