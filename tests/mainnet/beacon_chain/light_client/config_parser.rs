use std::collections::HashMap;
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

    fn process_value(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
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
        writeln!(file, "APP_NAME=myapp").unwrap();
        writeln!(file, "VERSION=1.0.0").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "  SPACED_KEY  =  spaced_value  ").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"myapp".to_string()));
        assert_eq!(config.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(config.get("SPACED_KEY"), Some(&"spaced_value".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("DB_HOST", "localhost");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE=${DB_HOST}").unwrap();
        writeln!(file, "STATIC_VALUE=production").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE"), Some(&"localhost".to_string()));
        assert_eq!(config.get("STATIC_VALUE"), Some(&"production".to_string()));
    }
}