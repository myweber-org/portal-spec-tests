use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub pool_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub workers: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug_mode: bool,
    pub log_level: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "".to_string(),
            database: "app_db".to_string(),
            pool_size: 10,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: "127.0.0.1".to_string(),
            port: 8080,
            workers: 4,
            timeout_seconds: 30,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            debug_mode: false,
            log_level: "info".to_string(),
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
                eprintln!("Failed to load config: {}. Using defaults.", e);
                AppConfig::default()
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        if self.database.port == 0 {
            return Err("Database port cannot be 0".to_string());
        }

        if self.database.pool_size == 0 {
            return Err("Database pool size cannot be 0".to_string());
        }

        if self.server.workers == 0 {
            return Err("Server workers cannot be 0".to_string());
        }

        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }

        Ok(())
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = self.to_toml()?;
        fs::write(path, toml_content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert!(!config.debug_mode);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());

        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = config.to_toml().unwrap();
        assert!(toml_str.contains("database"));
        assert!(toml_str.contains("server"));
    }

    #[test]
    fn test_config_file_operations() {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        config.save_to_file(path).unwrap();
        let loaded_config = AppConfig::load_from_file(path).unwrap();

        assert_eq!(config.database.host, loaded_config.database.host);
        assert_eq!(config.server.port, loaded_config.server.port);
        assert_eq!(config.debug_mode, loaded_config.debug_mode);
        assert_eq!(config.log_level, loaded_config.log_level);
    }
}use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub sections: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            sections: HashMap::new(),
        }
    }

    pub fn parse_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let mut config = Config::new();
        let mut current_section = String::from("default");
        config.sections.insert(current_section.clone(), HashMap::new());

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = trimmed[1..trimmed.len() - 1].trim().to_string();
                if section_name.is_empty() {
                    return Err(format!("Empty section name at line {}", line_num + 1));
                }
                current_section = section_name;
                config.sections.entry(current_section.clone()).or_insert_with(HashMap::new);
            } else {
                let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid key-value pair at line {}", line_num + 1));
                }

                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();

                if key.is_empty() {
                    return Err(format!("Empty key at line {}", line_num + 1));
                }

                config
                    .sections
                    .get_mut(&current_section)
                    .ok_or_else(|| format!("Section '{}' not found", current_section))?
                    .insert(key, value);
            }
        }

        Ok(config)
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section)?.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let content = r#"
# Sample configuration
server_host = 127.0.0.1
server_port = 8080

[database]
host = localhost
port = 5432
"#;

        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("default", "server_host"), Some(&"127.0.0.1".to_string()));
        assert_eq!(config.get("default", "server_port"), Some(&"8080".to_string()));
        assert_eq!(config.get("database", "host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("database", "port"), Some(&"5432".to_string()));
    }

    #[test]
    fn test_empty_section_name() {
        let content = "[]";
        let result = Config::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_value() {
        let content = "key_without_value";
        let result = Config::parse(content);
        assert!(result.is_err());
    }
}
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
                let processed_value = Self::process_value(value.trim());
                values.insert(key.trim().to_string(), processed_value);
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
        writeln!(file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some(&"postgres://localhost/test".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("API_KEY", "secret123");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "KEY=${API_KEY}").unwrap();
        writeln!(file, "NORMAL=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("KEY"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"value".to_string()));
    }
}