use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
    pub custom_settings: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let config: toml::Value = config_content.parse()
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        let database_url = Self::get_string(&config, "database.url")?;
        let api_key = env::var("API_KEY")
            .unwrap_or_else(|_| Self::get_string(&config, "api.key")?);
        let debug_mode = Self::get_bool(&config, "debug.enabled")?;
        let port = Self::get_u16(&config, "server.port")?;

        let custom_settings = if let Some(table) = config.get("custom") {
            Self::parse_custom_settings(table)
        } else {
            HashMap::new()
        };

        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
            custom_settings,
        })
    }

    fn get_string(config: &toml::Value, path: &str) -> Result<String, String> {
        let mut current = config;
        for part in path.split('.') {
            current = current.get(part)
                .ok_or_else(|| format!("Missing config key: {}", path))?;
        }
        current.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Config key {} is not a string", path))
    }

    fn get_bool(config: &toml::Value, path: &str) -> Result<bool, String> {
        let mut current = config;
        for part in path.split('.') {
            current = current.get(part)
                .ok_or_else(|| format!("Missing config key: {}", path))?;
        }
        current.as_bool()
            .ok_or_else(|| format!("Config key {} is not a boolean", path))
    }

    fn get_u16(config: &toml::Value, path: &str) -> Result<u16, String> {
        let mut current = config;
        for part in path.split('.') {
            current = current.get(part)
                .ok_or_else(|| format!("Missing config key: {}", path))?;
        }
        current.as_integer()
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| format!("Config key {} is not a valid u16", path))
    }

    fn parse_custom_settings(table: &toml::Value) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(table) = table.as_table() {
            for (key, value) in table {
                if let Some(str_val) = value.as_str() {
                    map.insert(key.clone(), str_val.to_string());
                }
            }
        }
        map
    }

    pub fn get_custom_setting(&self, key: &str) -> Option<&String> {
        self.custom_settings.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            [database]
            url = "postgresql://localhost/test"

            [api]
            key = "default_key"

            [debug]
            enabled = true

            [server]
            port = 8080

            [custom]
            feature_flag = "enabled"
            timeout = "30"
        "#;
        write!(temp_file, "{}", config_content).unwrap();

        env::set_var("CONFIG_PATH", temp_file.path());
        env::remove_var("API_KEY");

        let config = Config::load().unwrap();
        assert_eq!(config.database_url, "postgresql://localhost/test");
        assert_eq!(config.api_key, "default_key");
        assert!(config.debug_mode);
        assert_eq!(config.port, 8080);
        assert_eq!(config.get_custom_setting("feature_flag"), Some(&"enabled".to_string()));
    }

    #[test]
    fn test_env_override() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            [database]
            url = "postgresql://localhost/test"

            [api]
            key = "file_key"

            [debug]
            enabled = false

            [server]
            port = 3000
        "#;
        write!(temp_file, "{}", config_content).unwrap();

        env::set_var("CONFIG_PATH", temp_file.path());
        env::set_var("API_KEY", "env_key");

        let config = Config::load().unwrap();
        assert_eq!(config.api_key, "env_key");
        assert!(!config.debug_mode);
    }
}use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                self.values.insert(key, value);
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost/test".to_string()));
        assert_eq!(config.get("API_KEY"), Some("secret123".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_override() {
        env::set_var("SPECIAL_KEY", "env_value");
        let config = Config::new();
        assert_eq!(config.get("SPECIAL_KEY"), Some("env_value".to_string()));
    }

    #[test]
    fn test_default_value() {
        let config = Config::new();
        assert_eq!(config.get_with_default("MISSING", "default_value"), "default_value");
    }
}use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let mut config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.apply_env_overrides();
        config.validate()?;

        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(parsed_port) = port.parse::<u16>() {
                self.server_port = parsed_port;
            }
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level.to_uppercase();
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        let valid_log_levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }

        Ok(())
    }
}