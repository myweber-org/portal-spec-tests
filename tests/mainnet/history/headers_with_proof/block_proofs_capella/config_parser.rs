use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
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
                let processed_value = Self::process_value(value.trim());
                settings.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { settings })
    }

    fn process_value(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next(); // Skip '}'
                        break;
                    }
                    var_name.push(ch);
                    chars.next();
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
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "APP_NAME=MyApplication").unwrap();
        writeln!(temp_file, "VERSION=1.0.0").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "EMPTY_LINE=").unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"MyApplication".to_string()));
        assert_eq!(config.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(config.get("DATABASE_URL"), Some(&"postgres://localhost/db".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "HOST=${DB_HOST}").unwrap();
        writeln!(temp_file, "PORT=5432").unwrap();
        writeln!(temp_file, "CONNECTION=${{NOT_SET}}").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("CONNECTION"), Some(&"".to_string()));
    }
}