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
                let key = key.trim().to_string();
                let mut value = value.trim().to_string();

                if value.starts_with('$') {
                    let var_name = value.trim_start_matches('$');
                    if let Ok(env_value) = env::var(var_name) {
                        value = env_value;
                    }
                }

                values.insert(key, value);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "MAX_CONNECTIONS=10").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "API_KEY=$SECRET_KEY").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "10");
        assert_eq!(config.get_or_default("NON_EXISTENT", "default"), "default");
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

    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        for line in contents.lines() {
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

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.settings.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
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
    fn test_parse_valid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "host=localhost\nport=8080\n# This is a comment\n").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_get_or_default() {
        let mut config = Config::new();
        config.settings.insert("timeout".to_string(), "30".to_string());

        assert_eq!(config.get_or_default("timeout", "10"), "30");
        assert_eq!(config.get_or_default("retries", "3"), "3");
    }

    #[test]
    fn test_invalid_config_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid_line_without_equals").unwrap();

        let result = Config::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid config line"));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub features: HashMap<String, bool>,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&content)?;
        
        config.apply_environment_overrides();
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
        
        if let Ok(server_port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = server_port.parse::<u16>() {
                self.server.port = port_num;
            }
        }
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }
        
        if self.database.host.is_empty() {
            errors.push("Database host cannot be empty".to_string());
        }
        
        if self.database.port == 0 {
            errors.push("Database port cannot be zero".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!("Invalid log level: {}", self.log_level));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn load_config_with_fallback(paths: &[&str]) -> Result<AppConfig, Box<dyn std::error::Error>> {
    for path in paths {
        match AppConfig::from_file(path) {
            Ok(config) => return Ok(config),
            Err(e) => eprintln!("Failed to load config from {}: {}", path, e),
        }
    }
    
    Err("Could not load configuration from any provided path".into())
}use std::collections::HashMap;
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
        
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            
            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();
                
                value = self.substitute_env_vars(&value)?;
                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }
        }
        
        Ok(())
    }
    
    fn substitute_env_vars(&self, input: &str) -> Result<String, String> {
        let re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
        let mut result = input.to_string();
        
        for caps in re.captures_iter(input) {
            let var_name = &caps[1];
            match env::var(var_name) {
                Ok(val) => {
                    result = result.replace(&caps[0], &val);
                }
                Err(_) => {
                    return Err(format!("Environment variable '{}' not found", var_name));
                }
            }
        }
        
        Ok(result)
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
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = "APP_NAME=MyApp\nVERSION=1.0.0\nDEBUG=true";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("APP_NAME"), Some(&"MyApp".to_string()));
        assert_eq!(parser.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(parser.get("DEBUG"), Some(&"true".to_string()));
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("HOME_DIR", "/home/user");
        
        let mut parser = ConfigParser::new();
        let config = "DATA_PATH=${HOME_DIR}/data\nCACHE=${HOME_DIR}/cache";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("DATA_PATH"), Some(&"/home/user/data".to_string()));
        assert_eq!(parser.get("CACHE"), Some(&"/home/user/cache".to_string()));
    }
}use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub struct Config {
    sections: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            sections: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
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
                config.sections
                    .get_mut(&current_section)
                    .ok_or_else(|| format!("No section found for key '{}'", key))?
                    .insert(key, value);
            }
        }

        Ok(config)
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section)?.get(key)
    }

    pub fn sections(&self) -> Vec<&String> {
        self.sections.keys().collect()
    }

    pub fn keys_in_section(&self, section: &str) -> Option<Vec<&String>> {
        Some(self.sections.get(section)?.keys().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
# Sample config
[server]
host = 127.0.0.1
port = 8080

[database]
url = postgres://localhost/mydb
"#;
        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("server", "host"), Some(&"127.0.0.1".to_string()));
        assert_eq!(config.get("server", "port"), Some(&"8080".to_string()));
        assert_eq!(config.get("database", "url"), Some(&"postgres://localhost/mydb".to_string()));
    }

    #[test]
    fn test_default_section() {
        let content = r#"key1 = value1
key2 = value2"#;
        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("default", "key1"), Some(&"value1".to_string()));
        assert_eq!(config.get("default", "key2"), Some(&"value2".to_string()));
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
            let value = Self::resolve_value(&raw_value);
            
            values.insert(key, value);
        }
        
        Ok(Config { values })
    }
    
    fn resolve_value(raw_value: &str) -> String {
        if raw_value.starts_with("${") && raw_value.ends_with('}') {
            let env_var = &raw_value[2..raw_value.len() - 1];
            match env::var(env_var) {
                Ok(val) => val,
                Err(_) => raw_value.to_string(),
            }
        } else {
            raw_value.to_string()
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
    fn test_basic_config() {
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
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DB_HOST=localhost").unwrap();
        writeln!(file, "DB_PASS=${DB_PASSWORD}").unwrap();
        writeln!(file, "NO_ENV=${NONEXISTENT}").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DB_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DB_PASS"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NO_ENV"), Some(&"${NONEXISTENT}".to_string()));
    }
    
    #[test]
    fn test_invalid_format() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "INVALID_LINE").unwrap();
        
        let result = Config::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}