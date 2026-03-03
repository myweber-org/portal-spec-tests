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
        let contents = fs::read_to_string(path)?;
        let mut config_map = HashMap::new();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                config_map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = Self::get_value(map, "DATABASE_URL")?;
        let port = Self::get_value(map, "PORT")?.parse()?;
        let debug_mode = Self::get_value(map, "DEBUG")?.parse().unwrap_or(false);

        let mut api_keys = HashMap::new();
        for (key, value) in map {
            if key.starts_with("API_KEY_") {
                api_keys.insert(key.clone(), value.clone());
            }
        }

        Ok(Config {
            database_url,
            port,
            debug_mode,
            api_keys,
        })
    }

    fn get_value(map: &HashMap<String, String>, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        env::var(key)
            .ok()
            .or_else(|| map.get(key).cloned())
            .ok_or_else(|| format!("Missing configuration: {}", key).into())
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }

        if self.port == 0 {
            errors.push("PORT must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::substitute_env_vars(value.trim());
                settings.insert(key, processed_value);
            }
        }

        Ok(Config { settings })
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        for (key, env_value) in env::vars() {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, &env_value);
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
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
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "MAX_CONNECTIONS=10").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "10");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "8080");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PORT=${{APP_PORT}}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PORT").unwrap(), "8080");
    }
}