
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
            
            if let Some((key, value)) = trimmed.split_once('=') {
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
        assert_eq!(config.get("DATABASE_URL"), Some(&"postgres://localhost/db".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }
    
    #[test]
    fn test_env_interpolation() {
        env::set_var("API_KEY", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "API_KEY=$API_KEY").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("API_KEY"), Some(&"secret123".to_string()));
    }
}use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, io::Error> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                config.settings.insert(
                    key.trim().to_string(),
                    value.trim().trim_matches('"').to_string(),
                );
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.settings
            .get(key)
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
    fn test_empty_config() {
        let config = Config::new();
        assert!(config.settings.is_empty());
    }

    #[test]
    fn test_parse_config() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "# Sample config")?;
        writeln!(temp_file, "host=localhost")?;
        writeln!(temp_file, "port=8080")?;
        writeln!(temp_file, "debug=true")?;

        let config = Config::from_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("debug"), Some(&"true".to_string()));
        assert_eq!(config.get("missing"), None);

        Ok(())
    }

    #[test]
    fn test_get_or_default() {
        let mut config = Config::new();
        config.settings.insert("timeout".to_string(), "30".to_string());

        assert_eq!(config.get_or_default("timeout", "10"), "30");
        assert_eq!(config.get_or_default("retries", "3"), "3");
    }
}use std::fs;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config.settings.insert(key, value);
            } else {
                return Err(format!("Invalid config line: {}", line));
            }
        }

        Ok(config)
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.settings
            .get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost\nport=8080\n# This is a comment").unwrap();

        let config = Config::load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_with_default("host", "127.0.0.1"), "localhost");
        assert_eq!(config.get_with_default("port", "3000"), "8080");
    }

    #[test]
    fn test_load_invalid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid_line_without_equals").unwrap();

        let result = Config::load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_default_values() {
        let config = Config::new();
        assert_eq!(config.get_with_default("nonexistent", "default_value"), "default_value");
    }
}use std::collections::HashMap;
use std::fs;
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.parse_content(&content)
    }

    fn parse_content(&mut self, content: &str) -> Result<(), String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line format: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if value.is_empty() {
                return Err(format!("Empty value for key: {}", key));
            }

            self.settings.insert(key, value);
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }

    pub fn validate_required(&self, keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        for key in keys {
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
        writeln!(temp_file, "host=localhost\nport=8080\n# comment\n").unwrap();

        let result = config.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
    }

    #[test]
    fn test_missing_key() {
        let config = Config::new();
        assert_eq!(config.get("nonexistent"), None);
        assert_eq!(
            config.get_with_default("nonexistent", "default_value"),
            "default_value"
        );
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "required_key=value").unwrap();
        config.load_from_file(temp_file.path()).unwrap();

        let result = config.validate_required(&["required_key", "missing_key"]);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing, vec!["missing_key".to_string()]);
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
    pub file_path: String,
    pub max_size_mb: u64,
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
                file_path: "./logs/app.log".to_string(),
                max_size_mb: 100,
            },
        }
    }
}

pub fn load_config(config_path: &str) -> Result<AppConfig, String> {
    let path = Path::new(config_path);
    
    if !path.exists() {
        return Err(format!("Configuration file not found: {}", config_path));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    validate_config(&config)?;
    
    Ok(config)
}

pub fn load_config_or_default(config_path: &str) -> AppConfig {
    match load_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: Using default configuration. Error: {}", e);
            AppConfig::default()
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
    
    if config.logging.max_size_mb == 0 {
        return Err("Log file max size must be greater than 0".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_load_valid_config() {
        let config_content = r#"
            [server]
            host = "0.0.0.0"
            port = 9000
            timeout_seconds = 60

            [database]
            url = "postgresql://prod:5432/proddb"
            max_connections = 50
            min_connections = 10

            [logging]
            level = "debug"
            file_path = "/var/log/app.log"
            max_size_mb = 500
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", config_content).unwrap();
        
        let config = load_config(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.database.max_connections, 50);
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_load_invalid_config() {
        let config_content = r#"
            [server]
            host = "0.0.0.0"
            port = 0
            timeout_seconds = 60
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", config_content).unwrap();
        
        let result = load_config(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Server port cannot be 0"));
    }

    #[test]
    fn test_load_nonexistent_config() {
        let result = load_config("/nonexistent/path/config.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Configuration file not found"));
    }
}