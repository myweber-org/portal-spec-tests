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
}