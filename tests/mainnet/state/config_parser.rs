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
                let processed_value = Self::substitute_env_vars(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn substitute_env_vars(input: &str) -> String {
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
        self.values.get(key).cloned().unwrap_or(default.to_string())
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
        writeln!(file, "APP_NAME=myapp").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "  DEBUG=true  ").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"myapp".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://${DB_HOST}:5432/mydb").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(
            config.get("DATABASE_URL"),
            Some(&"postgres://localhost:5432/mydb".to_string())
        );
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
    pub cache_ttl: u64,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config_map = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                config_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }
        
        Self::from_map(&config_map)
    }
    
    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(map, "DATABASE_URL")
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/app".to_string());
        
        let server_port = Self::get_value(map, "SERVER_PORT")
            .and_then(|v| v.parse().ok())
            .or_else(|| env::var("SERVER_PORT").ok().and_then(|v| v.parse().ok()))
            .unwrap_or(8080);
        
        let log_level = Self::get_value(map, "LOG_LEVEL")
            .or_else(|| env::var("LOG_LEVEL").ok())
            .unwrap_or_else(|| "info".to_string());
        
        let cache_ttl = Self::get_value(map, "CACHE_TTL")
            .and_then(|v| v.parse().ok())
            .or_else(|| env::var("CACHE_TTL").ok().and_then(|v| v.parse().ok()))
            .unwrap_or(300);
        
        let mut features = HashMap::new();
        for (key, value) in map {
            if key.starts_with("FEATURE_") {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.to_lowercase() == "true" || value == "1";
                features.insert(feature_name, enabled);
            }
        }
        
        Ok(Config {
            database_url,
            server_port,
            log_level,
            cache_ttl,
            features,
        })
    }
    
    fn get_value(map: &HashMap<String, String>, key: &str) -> Option<String> {
        map.get(key).map(|s| s.to_string())
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
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
        writeln!(temp_file, "DATABASE_URL=postgres://prod:5432/db").unwrap();
        writeln!(temp_file, "SERVER_PORT=9090").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "FEATURE_API_V2=true").unwrap();
        writeln!(temp_file, "FEATURE_CACHE=false").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.database_url, "postgres://prod:5432/db");
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "info");
        assert!(config.is_feature_enabled("api_v2"));
        assert!(!config.is_feature_enabled("cache"));
    }
    
    #[test]
    fn test_env_fallback() {
        env::set_var("DATABASE_URL", "postgres://env:5432/db");
        
        let empty_map = HashMap::new();
        let config = Config::from_map(&empty_map).unwrap();
        
        assert_eq!(config.database_url, "postgres://env:5432/db");
        assert_eq!(config.server_port, 8080);
        
        env::remove_var("DATABASE_URL");
    }
}