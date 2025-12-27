use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                self.values.insert(key, value);
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                config.values.insert(key, value);
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key) || env::var(key).is_ok()
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                self.values.insert(key, value);
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_file() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost".to_string()));
        assert_eq!(config.get("API_KEY"), Some("secret123".to_string()));
    }

    #[test]
    fn test_environment_override() {
        env::set_var("OVERRIDE_KEY", "env_value");
        let config = Config::new();
        assert_eq!(config.get("OVERRIDE_KEY"), Some("env_value".to_string()));
    }
}