use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub thresholds: HashMap<String, f64>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        let mut thresholds = HashMap::new();

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
            let value = parts[1].trim().to_string();

            if key.starts_with("threshold_") {
                let num_value: f64 = value
                    .parse()
                    .map_err(|_| format!("Invalid number for key {}: {}", key, value))?;
                thresholds.insert(key, num_value);
            } else {
                settings.insert(key, value);
            }
        }

        Ok(Config {
            settings,
            thresholds,
        })
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_threshold(&self, key: &str) -> Option<&f64> {
        self.thresholds.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Sample config").unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "threshold_cpu=80.5").unwrap();
        writeln!(temp_file, "threshold_memory=90.0").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.get_setting("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get_setting("port"), Some(&"8080".to_string()));
        assert_eq!(config.get_threshold("threshold_cpu"), Some(&80.5));
        assert_eq!(config.get_threshold("threshold_memory"), Some(&90.0));
    }

    #[test]
    fn test_invalid_number() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "threshold_cpu=invalid").unwrap();

        let result = Config::from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
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

    pub fn get_with_fallback(&self, key: &str, fallback: &str) -> String {
        self.get(key).map(|s| s.as_str()).unwrap_or(fallback).to_string()
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
        write!(
            temp_file,
            "host=localhost\nport=8080\n# comment\n\ntimeout=60"
        )
        .unwrap();

        let result = config.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"60".to_string()));
        assert_eq!(config.get("retries"), Some(&"3".to_string()));
    }

    #[test]
    fn test_validation() {
        let config = Config::new();
        let result = config.validate_required(&["host", "port", "timeout"]);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert!(missing.contains(&"host".to_string()));
        assert!(missing.contains(&"port".to_string()));
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
                return Err(format!("Invalid line format: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::interpolate_env_vars(&raw_value);

            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn interpolate_env_vars(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();
        
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
    fn test_env_interpolation() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${{DB_PASSWORD}}").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
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
    pub connect_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size_mb: u64,
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
                max_connections: 10,
                min_connections: 2,
                connect_timeout: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "./logs/app.log".to_string(),
                max_file_size_mb: 100,
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

    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".into());
        }
        
        if self.database.max_connections < self.database.min_connections {
            return Err("Max connections cannot be less than min connections".into());
        }
        
        if self.logging.max_file_size_mb == 0 {
            return Err("Max file size must be greater than zero".into());
        }
        
        Ok(())
    }
    
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
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
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = line.split_once('=') {
                let key = key.trim().to_string();
                value = value.trim();

                let mut processed_value = value.to_string();
                for cap in var_pattern.captures_iter(value) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
            host=localhost
            port=8080
            # This is a comment
            timeout=30
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "postgres-server");
        env::set_var("API_PORT", "9000");

        let content = r#"
            database_host=${DB_HOST}
            api_port=${API_PORT}
            static_value=constant
            mixed=server-${DB_HOST}-${API_PORT}
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("database_host"), Some(&"postgres-server".to_string()));
        assert_eq!(config.get("api_port"), Some(&"9000".to_string()));
        assert_eq!(config.get("static_value"), Some(&"constant".to_string()));
        assert_eq!(config.get("mixed"), Some(&"server-postgres-server-9000".to_string()));
    }

    #[test]
    fn test_file_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "key1=value1\nkey2=value2").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("key1"), Some(&"value1".to_string()));
        assert_eq!(config.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let content = "existing=found";
        let config = Config::from_str(content).unwrap();
        
        assert_eq!(config.get_or_default("existing", "default"), "found");
        assert_eq!(config.get_or_default("missing", "default"), "default");
    }
}