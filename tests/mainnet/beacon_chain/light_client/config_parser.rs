
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::substitute_env_vars(&raw_value);

            settings.insert(key, value);
        }

        Ok(Config { settings })
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        
        for (key, env_value) in env::vars() {
            let placeholder = format!("${}", key);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &env_value);
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
        writeln!(temp_file, "HOST=localhost").unwrap();
        writeln!(temp_file, "PORT=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_ENV", "production");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ENVIRONMENT=$APP_ENV").unwrap();
        writeln!(temp_file, "PATH=/opt/$APP_ENV/bin").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("ENVIRONMENT"), Some(&"production".to_string()));
        assert_eq!(config.get("PATH"), Some(&"/opt/production/bin".to_string()));
    }
}