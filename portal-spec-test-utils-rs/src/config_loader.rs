use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                config.apply_setting(key, value);
            }
        }
        
        Ok(config)
    }
    
    fn apply_setting(&mut self, key: &str, value: &str) {
        match key {
            "DATABASE_URL" => self.database_url = Self::resolve_env_var(value),
            "SERVER_PORT" => {
                if let Ok(port) = value.parse() {
                    self.server_port = port;
                }
            }
            "LOG_LEVEL" => self.log_level = value.to_string(),
            _ if key.starts_with("FEATURE_") => {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.eq_ignore_ascii_case("true");
                self.features.insert(feature_name, enabled);
            }
            _ => {}
        }
    }
    
    fn resolve_env_var(value: &str) -> String {
        if let Some(var_name) = value.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: String::from("postgres://localhost:5432/db"),
            server_port: 8080,
            log_level: String::from("info"),
            features: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let config_content = r#"
            DATABASE_URL=postgres://user:pass@localhost:5432/app
            SERVER_PORT=3000
            LOG_LEVEL=debug
            FEATURE_API_V2=true
            FEATURE_CACHE=false
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::io::write(&mut file, config_content).unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.database_url, "postgres://user:pass@localhost:5432/app");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "debug");
        assert!(config.is_feature_enabled("api_v2"));
        assert!(!config.is_feature_enabled("cache"));
    }
    
    #[test]
    fn test_env_var_resolution() {
        env::set_var("DB_HOST", "database.server.com");
        
        let config_content = "DATABASE_URL=postgres://${DB_HOST}:5432/db";
        let mut file = NamedTempFile::new().unwrap();
        std::io::write(&mut file, config_content).unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://database.server.com:5432/db");
        
        env::remove_var("DB_HOST");
    }
}