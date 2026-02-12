use std::collections::HashMap;
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

impl Default for Config {
    fn default() -> Self {
        Config {
            database_url: String::from("postgresql://localhost:5432/db"),
            max_connections: 10,
            timeout_seconds: 30,
            features: vec![String::from("logging"), String::from("cache")],
            metadata: HashMap::new(),
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::default();
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
                let key = trimmed[..equal_pos].trim();
                let value = trimmed[equal_pos+1..].trim();
                
                match current_section.as_str() {
                    "database" => Self::parse_database_field(&mut config, key, value, line_num)?,
                    "features" => Self::parse_features_field(&mut config, key, value, line_num)?,
                    "metadata" => Self::parse_metadata_field(&mut config, key, value, line_num)?,
                    _ => return Err(format!("Unknown section '{}' at line {}", current_section, line_num + 1)),
                }
            } else {
                return Err(format!("Invalid config format at line {}", line_num + 1));
            }
        }

        config.validate()?;
        Ok(config)
    }

    fn parse_database_field(config: &mut Config, key: &str, value: &str, line_num: usize) -> Result<(), String> {
        match key {
            "url" => config.database_url = value.to_string(),
            "max_connections" => {
                config.max_connections = value.parse()
                    .map_err(|_| format!("Invalid integer for max_connections at line {}", line_num + 1))?
            }
            "timeout" => {
                config.timeout_seconds = value.parse()
                    .map_err(|_| format!("Invalid integer for timeout at line {}", line_num + 1))?
            }
            _ => return Err(format!("Unknown database field '{}' at line {}", key, line_num + 1)),
        }
        Ok(())
    }

    fn parse_features_field(config: &mut Config, key: &str, value: &str, line_num: usize) -> Result<(), String> {
        if key != "enabled" {
            return Err(format!("Unknown features field '{}' at line {}", key, line_num + 1));
        }
        
        config.features = value.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(())
    }

    fn parse_metadata_field(config: &mut Config, key: &str, value: &str, _line_num: usize) -> Result<(), String> {
        config.metadata.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        
        if self.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }
        
        if self.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
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
        let config = Config::default();
        assert_eq!(config.database_url, "postgresql://localhost:5432/db");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.get_feature_status("logging"));
        assert!(config.get_feature_status("cache"));
    }

    #[test]
    fn test_valid_config_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[database]").unwrap();
        writeln!(file, "url = postgresql://prod:5432/app").unwrap();
        writeln!(file, "max_connections = 20").unwrap();
        writeln!(file, "timeout = 60").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "[features]").unwrap();
        writeln!(file, "enabled = logging,cache,metrics").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "[metadata]").unwrap();
        writeln!(file, "version = 1.2.3").unwrap();
        writeln!(file, "environment = production").unwrap();

        let config = Config::from_file(file.path()).unwrap();
        assert_eq!(config.database_url, "postgresql://prod:5432/app");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.timeout_seconds, 60);
        assert!(config.get_feature_status("metrics"));
        assert_eq!(config.get_metadata("version"), Some(&"1.2.3".to_string()));
    }

    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[database]").unwrap();
        writeln!(file, "max_connections = not_a_number").unwrap();

        let result = Config::from_file(file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid integer"));
    }

    #[test]
    fn test_missing_file() {
        let result = Config::from_file("nonexistent_file.conf");
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
            let value = Self::substitute_env_vars(&raw_value);
            
            values.insert(key, value);
        }
        
        Ok(Config { values })
    }
    
    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        
        for (key, env_value) in env::vars() {
            let placeholder = format!("${}", key);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &env_value);
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
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    pub fn len(&self) -> usize {
        self.values.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
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
        writeln!(file, "TIMEOUT=30").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST").unwrap(), "localhost");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert_eq!(config.len(), 3);
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "postgres-server");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://${DB_HOST}:5432/mydb").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://postgres-server:5432/mydb");
        
        env::remove_var("DB_HOST");
    }
    
    #[test]
    fn test_invalid_format() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "INVALID_LINE").unwrap();
        
        let result = Config::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING_KEY=value").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING_KEY", "default"), "value");
        assert_eq!(config.get_or_default("NON_EXISTENT", "default"), "default");
    }
}