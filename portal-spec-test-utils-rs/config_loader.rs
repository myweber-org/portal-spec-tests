use std::env;
use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let mut config = HashMap::new();
        
        if let Ok(contents) = fs::read_to_string("config.toml") {
            for line in contents.lines() {
                if line.trim().is_empty() || line.starts_with('#') {
                    continue;
                }
                
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    config.insert(
                        parts[0].trim().to_string(),
                        parts[1].trim().to_string()
                    );
                }
            }
        }
        
        let database_url = env::var("DATABASE_URL")
            .ok()
            .or_else(|| config.get("database_url").cloned())
            .ok_or("Database URL not found in config or environment")?;
            
        let api_key = env::var("API_KEY")
            .ok()
            .or_else(|| config.get("api_key").cloned())
            .ok_or("API key not found in config or environment")?;
            
        let debug_mode = env::var("DEBUG_MODE")
            .ok()
            .and_then(|v| v.parse().ok())
            .or_else(|| config.get("debug_mode").and_then(|v| v.parse().ok()))
            .unwrap_or(false);
            
        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .or_else(|| config.get("port").and_then(|v| v.parse().ok()))
            .unwrap_or(8080);
            
        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
        })
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }
        
        if self.api_key.len() < 32 {
            errors.push("API key must be at least 32 characters".to_string());
        }
        
        if self.port == 0 || self.port > 65535 {
            errors.push(format!("Port {} is invalid", self.port));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}