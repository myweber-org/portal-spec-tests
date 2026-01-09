
use std::collections::HashMap;
use std::env;
use std::fs;
use regex::Regex;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut values = HashMap::new();
        let re = Regex::new(r"^([A-Za-z0-9_]+)\s*=\s*(.+)$").unwrap();
        let var_re = Regex::new(r"\$\{([A-Za-z0-9_]+)\}").unwrap();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();
                
                // Replace environment variables
                value = var_re.replace_all(&value, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    env::var(var_name).unwrap_or_else(|_| String::new())
                }).to_string();
                
                values.insert(key, value);
            }
        }
        
        Ok(Config { values })
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
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

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
                let key = key.trim().to_string();
                let processed_value = Self::substitute_env_vars(value.trim());
                settings.insert(key, processed_value);
            }
        }

        Ok(Config { settings })
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        for (key, env_value) in env::vars() {
            let placeholder = format!("${}", key);
            result = result.replace(&placeholder, &env_value);
        }
        result
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
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("API_KEY").unwrap(), "secret123");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("HOME", "/users/test");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATA_PATH=$HOME/data").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATA_PATH").unwrap(), "/users/test/data");
    }
}
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
    pub database_name: String,
}

#[derive(Debug, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|_| ConfigError::FileNotFound(path.to_string()))?;

        let parsed: toml::Value = content
            .parse()
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Self::validate_and_build(parsed)
    }

    fn validate_and_build(value: toml::Value) -> Result<Self, ConfigError> {
        let table = value.as_table()
            .ok_or_else(|| ConfigError::ValidationError("Root must be a table".to_string()))?;

        let database = Self::parse_database(table)?;
        let server = Self::parse_server(table)?;
        let features = Self::parse_features(table)?;

        Ok(Config {
            database,
            server,
            features,
        })
    }

    fn parse_database(table: &toml::value::Table) -> Result<DatabaseConfig, ConfigError> {
        let db_table = table.get("database")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::ValidationError("Missing database section".to_string()))?;

        let host = db_table.get("host")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConfigError::ValidationError("Database host is required".to_string()))?;

        let port = db_table.get("port")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| ConfigError::ValidationError("Database port is required".to_string()))?;

        if port < 1 || port > 65535 {
            return Err(ConfigError::ValidationError(
                "Database port must be between 1 and 65535".to_string()
            ));
        }

        let username = db_table.get("username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConfigError::ValidationError("Database username is required".to_string()))?;

        let password = db_table.get("password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConfigError::ValidationError("Database password is required".to_string()))?;

        let database_name = db_table.get("database_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConfigError::ValidationError("Database name is required".to_string()))?;

        Ok(DatabaseConfig {
            host,
            port: port as u16,
            username,
            password,
            database_name,
        })
    }

    fn parse_server(table: &toml::value::Table) -> Result<ServerConfig, ConfigError> {
        let server_table = table.get("server")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::ValidationError("Missing server section".to_string()))?;

        let host = server_table.get("host")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConfigError::ValidationError("Server host is required".to_string()))?;

        let port = server_table.get("port")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| ConfigError::ValidationError("Server port is required".to_string()))?;

        if port < 1 || port > 65535 {
            return Err(ConfigError::ValidationError(
                "Server port must be between 1 and 65535".to_string()
            ));
        }

        let max_connections = server_table.get("max_connections")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| ConfigError::ValidationError("Max connections is required".to_string()))?;

        if max_connections < 1 {
            return Err(ConfigError::ValidationError(
                "Max connections must be at least 1".to_string()
            ));
        }

        let timeout_seconds = server_table.get("timeout_seconds")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| ConfigError::ValidationError("Timeout seconds is required".to_string()))?;

        if timeout_seconds < 1 {
            return Err(ConfigError::ValidationError(
                "Timeout seconds must be at least 1".to_string()
            ));
        }

        Ok(ServerConfig {
            host,
            port: port as u16,
            max_connections: max_connections as u32,
            timeout_seconds: timeout_seconds as u64,
        })
    }

    fn parse_features(table: &toml::value::Table) -> Result<HashMap<String, bool>, ConfigError> {
        let mut features = HashMap::new();

        if let Some(features_table) = table.get("features").and_then(|v| v.as_table()) {
            for (key, value) in features_table {
                if let Some(bool_val) = value.as_bool() {
                    features.insert(key.clone(), bool_val);
                } else {
                    return Err(ConfigError::ValidationError(
                        format!("Feature '{}' must be a boolean value", key)
                    ));
                }
            }
        }

        Ok(features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            database_name = "app_db"

            [server]
            host = "0.0.0.0"
            port = 8080
            max_connections = 100
            timeout_seconds = 30

            [features]
            logging = true
            caching = false
        "#;

        let parsed: toml::Value = toml_content.parse().unwrap();
        let config = Config::validate_and_build(parsed).unwrap();

        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.max_connections, 100);
        assert_eq!(config.features.get("logging"), Some(&true));
        assert_eq!(config.features.get("caching"), Some(&false));
    }

    #[test]
    fn test_missing_section() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            database_name = "app_db"
        "#;

        let parsed: toml::Value = toml_content.parse().unwrap();
        let result = Config::validate_and_build(parsed);
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
    }

    #[test]
    fn test_invalid_port() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 70000
            username = "admin"
            password = "secret"
            database_name = "app_db"

            [server]
            host = "0.0.0.0"
            port = 8080
            max_connections = 100
            timeout_seconds = 30
        "#;

        let parsed: toml::Value = toml_content.parse().unwrap();
        let result = Config::validate_and_build(parsed);
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
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
                let key = key.trim().to_string();
                let value = Self::resolve_value(value.trim());
                values.insert(key, value);
            }
        }

        Ok(Config { values })
    }

    fn resolve_value(input: &str) -> String {
        if let Some(var_name) = input.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
            env::var(var_name).unwrap_or_else(|_| input.to_string())
        } else {
            input.to_string()
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
    fn test_env_variable_expansion() {
        env::set_var("DB_PASSWORD", "secret123");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${DB_PASSWORD}").unwrap();
        writeln!(file, "NORMAL=plain_value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"plain_value".to_string()));
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn new() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let mut config = Self::default();
        
        if let Ok(content) = fs::read_to_string(&config_path) {
            config.parse_toml(&content)?;
        }
        
        config.apply_environment_overrides();
        
        Ok(config)
    }
    
    fn parse_toml(&mut self, content: &str) -> Result<(), String> {
        let parsed: toml::Value = content.parse()
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;
        
        if let Some(table) = parsed.as_table() {
            if let Some(url) = table.get("database_url").and_then(|v| v.as_str()) {
                self.database_url = url.to_string();
            }
            
            if let Some(port) = table.get("port").and_then(|v| v.as_integer()) {
                self.port = port as u16;
            }
            
            if let Some(level) = table.get("log_level").and_then(|v| v.as_str()) {
                self.log_level = level.to_string();
            }
            
            if let Some(features) = table.get("features").and_then(|v| v.as_table()) {
                for (key, value) in features {
                    if let Some(bool_val) = value.as_bool() {
                        self.features.insert(key.clone(), bool_val);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(url) = env::var("DATABASE_URL") {
            self.database_url = url;
        }
        
        if let Ok(port) = env::var("PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.port = port_num;
            }
        }
        
        if let Ok(level) = env::var("LOG_LEVEL") {
            self.log_level = level;
        }
        
        for (key, _) in env::vars() {
            if key.starts_with("FEATURE_") {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = env::var(&key)
                    .map(|v| v.to_lowercase() == "true" || v == "1")
                    .unwrap_or(false);
                self.features.insert(feature_name, enabled);
            }
        }
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost:5432/app".to_string(),
            port: 8080,
            log_level: "info".to_string(),
            features: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database_url, "postgres://localhost:5432/app");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
    }
    
    #[test]
    fn test_feature_check() {
        let mut config = Config::default();
        config.features.insert("dark_mode".to_string(), true);
        config.features.insert("experimental".to_string(), false);
        
        assert!(config.is_feature_enabled("dark_mode"));
        assert!(!config.is_feature_enabled("experimental"));
        assert!(!config.is_feature_enabled("nonexistent"));
    }
}