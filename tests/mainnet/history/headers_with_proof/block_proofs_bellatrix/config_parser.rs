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
            let mut value = parts[1].trim().to_string();
            
            value = Self::expand_env_vars(&value);
            values.insert(key, value);
        }
        
        Ok(Config { values })
    }
    
    fn expand_env_vars(input: &str) -> String {
        let mut result = input.to_string();
        
        for (key, val) in env::vars() {
            let placeholder = format!("${}", key);
            result = result.replace(&placeholder, &val);
        }
        
        result
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
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
        writeln!(file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/test");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(config.get("NONEXISTENT").is_none());
    }
    
    #[test]
    fn test_env_var_expansion() {
        env::set_var("APP_ENV", "production");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "ENVIRONMENT=${APP_ENV}").unwrap();
        writeln!(file, "PATH=/home/${USER}/data").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("ENVIRONMENT").unwrap(), "production");
    }
    
    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}