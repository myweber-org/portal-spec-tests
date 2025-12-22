use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
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
                config.parse_key_value(key, value);
            }
        }
        
        Ok(config)
    }
    
    fn parse_key_value(&mut self, key: &str, value: &str) {
        match key {
            "DATABASE_URL" => self.database_url = Self::resolve_env_var(value).to_string(),
            "PORT" => {
                if let Ok(port) = value.parse() {
                    self.port = port;
                }
            }
            "DEBUG_MODE" => self.debug_mode = value.parse().unwrap_or(false),
            key if key.starts_with("API_KEY_") => {
                let service = key.trim_start_matches("API_KEY_").to_lowercase();
                self.api_keys.insert(service, Self::resolve_env_var(value).to_string());
            }
            _ => {}
        }
    }
    
    fn resolve_env_var(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len()-1];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
    
    pub fn default() -> Self {
        Self {
            database_url: "postgres://localhost:5432/mydb".to_string(),
            port: 8080,
            debug_mode: false,
            api_keys: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let config_content = r#"
            DATABASE_URL=postgres://user:pass@localhost/db
            PORT=3000
            DEBUG_MODE=true
            API_KEY_WEATHER=${WEATHER_API_KEY}
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_content).unwrap();
        
        env::set_var("WEATHER_API_KEY", "test_key_123");
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.port, 3000);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.api_keys.get("weather"), Some(&"test_key_123".to_string()));
    }
}