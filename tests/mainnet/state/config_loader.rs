use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "DATABASE_URL" => config.database_url = Self::resolve_env_var(value),
                    "PORT" => config.port = value.parse().unwrap_or(8080),
                    "DEBUG_MODE" => config.debug_mode = value.parse().unwrap_or(false),
                    _ if key.starts_with("API_KEY_") => {
                        let service = key.trim_start_matches("API_KEY_").to_lowercase();
                        config.api_keys.insert(service, Self::resolve_env_var(value));
                    }
                    _ => {}
                }
            }
        }
        
        Ok(config)
    }
    
    fn resolve_env_var(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: String::from("postgres://localhost:5432/db"),
            port: 8080,
            debug_mode: false,
            api_keys: HashMap::new(),
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
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://prod:5432/app").unwrap();
        writeln!(file, "PORT=9000").unwrap();
        writeln!(file, "DEBUG_MODE=true").unwrap();
        writeln!(file, "API_KEY_WEATHER=${WEATHER_API_KEY}").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        
        env::set_var("WEATHER_API_KEY", "secret-123");
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://prod:5432/app");
        assert_eq!(config.port, 9000);
        assert!(config.debug_mode);
        assert_eq!(config.api_keys.get("weather"), Some(&"secret-123".to_string()));
    }
}