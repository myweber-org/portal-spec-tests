
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut values = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::interpolate_env_vars(&raw_value);

            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn interpolate_env_vars(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
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
                
                if !var_name.is_empty() {
                    result.push_str(&env::var(&var_name).unwrap_or_default());
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
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST").unwrap(), "localhost");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("APP_ENV", "production");
        env::set_var("DB_HOST", "db.example.com");

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "ENVIRONMENT=${APP_ENV}").unwrap();
        writeln!(file, "DATABASE_HOST=${DB_HOST}").unwrap();
        writeln!(file, "NESTED=prefix_${APP_ENV}_suffix").unwrap();
        writeln!(file, "UNDEFINED=${NOT_SET}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("ENVIRONMENT").unwrap(), "production");
        assert_eq!(config.get("DATABASE_HOST").unwrap(), "db.example.com");
        assert_eq!(config.get("NESTED").unwrap(), "prefix_production_suffix");
        assert_eq!(config.get("UNDEFINED").unwrap(), "");
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SET_VALUE=actual").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("SET_VALUE", "default"), "actual");
        assert_eq!(config.get_or_default("UNSET_VALUE", "default"), "default");
    }
}