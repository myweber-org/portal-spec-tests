use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    ParseError(String),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut config = Config::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(ConfigError::ParseError(format!(
                    "Invalid format at line {}: {}",
                    line_num + 1,
                    trimmed
                )));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if key.is_empty() {
                return Err(ConfigError::ParseError(format!(
                    "Empty key at line {}",
                    line_num + 1
                )));
            }

            config.values.insert(key, value);
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
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
    fn test_load_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "timeout=30").unwrap();

        let config = Config::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.len(), 3);
    }

    #[test]
    fn test_load_invalid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "key_without_value=").unwrap();

        let result = Config::load_from_file(temp_file.path());
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }

    #[test]
    fn test_empty_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config::load_from_file(temp_file.path()).unwrap();
        assert!(config.is_empty());
    }
}use std::collections::HashMap;
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
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map_or(default.to_string(), |v| v.clone())
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
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "super-secret-key");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET_KEY=${APP_SECRET}").unwrap();
        writeln!(file, "HOST=localhost:${PORT}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET_KEY").unwrap(), "super-secret-key");
        assert_eq!(config.get("HOST").unwrap(), "localhost:");
    }
}