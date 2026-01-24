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
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        
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
                
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
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
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("PORT").unwrap(), "8080");
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "mysecret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET_KEY=${APP_SECRET}").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET_KEY").unwrap(), "mysecret123");
    }
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

    pub fn parse_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    pub fn parse_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let env_var_regex = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                
                for capture in env_var_regex.captures_iter(&value) {
                    if let Some(var_name) = capture.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            value = value.replace(&capture[0], &env_value);
                        }
                    }
                }
                
                self.values.insert(key, value.trim().to_string());
            }
        }
        
        Ok(())
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

    pub fn all_values(&self) -> &HashMap<String, String> {
        &self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let content = "HOST=localhost\nPORT=8080\nDEBUG=true\n";
        
        parser.parse_content(content).unwrap();
        
        assert_eq!(parser.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(parser.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(parser.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "postgres-server");
        
        let mut parser = ConfigParser::new();
        let content = "DATABASE_HOST=${DB_HOST}\nDATABASE_PORT=5432\n";
        
        parser.parse_content(content).unwrap();
        
        assert_eq!(parser.get("DATABASE_HOST"), Some(&"postgres-server".to_string()));
        assert_eq!(parser.get("DATABASE_PORT"), Some(&"5432".to_string()));
    }

    #[test]
    fn test_file_parsing() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.cfg");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "APP_NAME=TestApp\nVERSION=1.0.0").unwrap();
        
        let mut parser = ConfigParser::new();
        parser.parse_file(file_path.to_str().unwrap()).unwrap();
        
        assert_eq!(parser.get("APP_NAME"), Some(&"TestApp".to_string()));
        assert_eq!(parser.get("VERSION"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut parser = ConfigParser::new();
        let content = "EXISTING_KEY=actual_value\n";
        
        parser.parse_content(content).unwrap();
        
        assert_eq!(parser.get_or_default("EXISTING_KEY", "default"), "actual_value");
        assert_eq!(parser.get_or_default("NON_EXISTENT", "default_value"), "default_value");
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
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(raw: &str) -> String {
        if raw.starts_with('$') {
            let var_name = &raw[1..];
            env::var(var_name).unwrap_or_else(|_| raw.to_string())
        } else {
            raw.to_string()
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
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "my_secret_key");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET=$APP_SECRET").unwrap();
        writeln!(file, "NORMAL=plain_value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET"), Some(&"my_secret_key".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"plain_value".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=found").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "found");
        assert_eq!(config.get_or_default("MISSING", "default_value"), "default_value");
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
            let var_name = &raw_value[2..raw_value.len() - 1];
            match env::var(var_name) {
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
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub features: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            database_url: String::from("postgresql://localhost:5432"),
            max_connections: 10,
            timeout_seconds: 30,
            features: vec![],
            metadata: HashMap::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        let mut current_section = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len()-1].to_string();
                continue;
            }

            if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let value = trimmed[equal_pos+1..].trim().to_string();

                match (current_section.as_str(), key.as_str()) {
                    ("database", "url") => config.database_url = value,
                    ("database", "max_connections") => {
                        config.max_connections = value.parse()
                            .map_err(|_| format!("Line {}: Invalid integer for max_connections", line_num + 1))?
                    }
                    ("network", "timeout") => {
                        config.timeout_seconds = value.parse()
                            .map_err(|_| format!("Line {}: Invalid integer for timeout", line_num + 1))?
                    }
                    ("features", _) => config.features.push(value),
                    ("metadata", _) => {
                        config.metadata.insert(key, value);
                    }
                    _ => return Err(format!("Line {}: Unknown configuration key '{}' in section '{}'", 
                        line_num + 1, key, current_section)),
                }
            } else {
                return Err(format!("Line {}: Invalid configuration line", line_num + 1));
            }
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        if self.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }

        if self.timeout_seconds > 3600 {
            return Err("Timeout cannot exceed 3600 seconds".to_string());
        }

        Ok(())
    }

    pub fn get_feature_status(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert_eq!(config.database_url, "postgresql://localhost:5432");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.features.is_empty());
    }

    #[test]
    fn test_config_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[database]").unwrap();
        writeln!(file, "url = postgresql://remote:5432").unwrap();
        writeln!(file, "max_connections = 20").unwrap();
        writeln!(file, "[network]").unwrap();
        writeln!(file, "timeout = 60").unwrap();
        writeln!(file, "[features]").unwrap();
        writeln!(file, "logging").unwrap();
        writeln!(file, "caching").unwrap();
        writeln!(file, "[metadata]").unwrap();
        writeln!(file, "version = 1.0.0").unwrap();

        let config = Config::from_file(file.path()).unwrap();
        assert_eq!(config.database_url, "postgresql://remote:5432");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.features, vec!["logging", "caching"]);
        assert_eq!(config.get_metadata("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        assert!(config.validate().is_ok());

        config.database_url = String::new();
        assert!(config.validate().is_err());

        config.database_url = "postgresql://localhost:5432".to_string();
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config.max_connections = 10;
        config.timeout_seconds = 4000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_feature_check() {
        let mut config = Config::new();
        config.features = vec!["logging".to_string(), "metrics".to_string()];
        
        assert!(config.get_feature_status("logging"));
        assert!(config.get_feature_status("metrics"));
        assert!(!config.get_feature_status("debug"));
    }
}