
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut config_map = HashMap::new();

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                config_map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Self::from_map(config_map)
    }

    fn from_map(mut map: HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = Self::get_value(&mut map, "DATABASE_URL")?;
        let server_port = Self::get_value(&mut map, "SERVER_PORT")?.parse()?;
        let log_level = Self::get_value(&mut map, "LOG_LEVEL")?;
        let cache_ttl = Self::get_value(&mut map, "CACHE_TTL")?.parse()?;

        Ok(Config {
            database_url,
            server_port,
            log_level,
            cache_ttl,
        })
    }

    fn get_value(map: &mut HashMap<String, String>, key: &str) -> Result<String, String> {
        if let Ok(env_value) = env::var(key) {
            return Ok(env_value);
        }

        map.remove(key)
            .ok_or_else(|| format!("Missing configuration key: {}", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(temp_file, "SERVER_PORT=8080").unwrap();
        writeln!(temp_file, "LOG_LEVEL=info").unwrap();
        writeln!(temp_file, "CACHE_TTL=300").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_env_override() {
        env::set_var("DATABASE_URL", "postgres://prod/db");
        
        let mut map = HashMap::new();
        map.insert("DATABASE_URL".to_string(), "postgres://localhost/test".to_string());
        map.insert("SERVER_PORT".to_string(), "8080".to_string());
        map.insert("LOG_LEVEL".to_string(), "info".to_string());
        map.insert("CACHE_TTL".to_string(), "300".to_string());

        let config = Config::from_map(map).unwrap();
        assert_eq!(config.database_url, "postgres://prod/db");
        
        env::remove_var("DATABASE_URL");
    }
}