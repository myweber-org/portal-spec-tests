use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub cache_size: usize,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let parsed = parse_config(&content)?;
        
        Ok(Config {
            database_url: get_value(&parsed, "DATABASE_URL")?,
            port: get_value(&parsed, "PORT")?.parse()?,
            log_level: get_value(&parsed, "LOG_LEVEL")?,
            cache_size: get_value(&parsed, "CACHE_SIZE")?.parse()?,
        })
    }
}

fn parse_config(content: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut config = HashMap::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = substitute_env_vars(value.trim());
            config.insert(key, value);
        }
    }
    
    Ok(config)
}

fn substitute_env_vars(value: &str) -> String {
    let mut result = value.to_string();
    
    for (key, env_value) in env::vars() {
        let placeholder = format!("${}", key);
        result = result.replace(&placeholder, &env_value);
    }
    
    result
}

fn get_value(config: &HashMap<String, String>, key: &str) -> Result<String, Box<dyn std::error::Error>> {
    config.get(key)
        .cloned()
        .ok_or_else(|| format!("Missing configuration key: {}", key).into())
}