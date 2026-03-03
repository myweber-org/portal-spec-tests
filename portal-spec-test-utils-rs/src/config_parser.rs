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
                let processed_value = Self::interpolate_env_vars(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn interpolate_env_vars(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
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
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
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
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "MAX_CONNECTIONS=100").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "100");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("APP_PORT", "8080");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PORT=${APP_PORT}").unwrap();
        writeln!(file, "HOST=localhost:${APP_PORT}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("HOST").unwrap(), "localhost:8080");
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub max_connections: u32,
    pub debug_mode: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::resolve_env_vars(&raw_value);
            config_map.insert(key, value);
        }

        Ok(Config {
            database_url: config_map
                .get("DATABASE_URL")
                .ok_or("Missing DATABASE_URL")?
                .clone(),
            api_key: config_map
                .get("API_KEY")
                .ok_or("Missing API_KEY")?
                .clone(),
            max_connections: config_map
                .get("MAX_CONNECTIONS")
                .unwrap_or(&"10".to_string())
                .parse()
                .map_err(|_| "Invalid MAX_CONNECTIONS value")?,
            debug_mode: config_map
                .get("DEBUG_MODE")
                .unwrap_or(&"false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }

    fn resolve_env_vars(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                
                match env::var(&var_name) {
                    Ok(env_value) => result.push_str(&env_value),
                    Err(_) => result.push_str(&format!("${{{}}}", var_name)),
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(config_file, "API_KEY=secret123").unwrap();
        writeln!(config_file, "MAX_CONNECTIONS=20").unwrap();
        writeln!(config_file, "DEBUG_MODE=true").unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.api_key, "secret123");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.debug_mode, true);
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("DB_HOST", "localhost");
        
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "DATABASE_URL=postgres://${DB_HOST}/db").unwrap();
        writeln!(config_file, "API_KEY=test").unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
    }
}
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
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }
        
        Ok(Config { values })
    }
    
    fn process_value(value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
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
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "DEBUG=true").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("DEBUG").unwrap(), "true");
        assert!(config.get("NONEXISTENT").is_none());
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("API_KEY", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "API_KEY=$API_KEY").unwrap();
        writeln!(file, "OTHER=$UNDEFINED_VAR").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("API_KEY").unwrap(), "secret123");
        assert_eq!(config.get("OTHER").unwrap(), "$UNDEFINED_VAR");
    }
    
    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
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
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
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
        self.values.get(key).cloned().unwrap_or(default.to_string())
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
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${{DB_PASSWORD}}").unwrap();
        writeln!(file, "URL=postgres://user:${{DB_PASSWORD}}@localhost").unwrap();
        writeln!(file, "MISSING=${{UNDEFINED_VAR}}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("URL"), Some(&"postgres://user:secret123@localhost".to_string()));
        assert_eq!(config.get("MISSING"), Some(&"${UNDEFINED_VAR}".to_string()));
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                config_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(map, "DATABASE_URL")
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/app".to_string());

        let port = Self::get_value(map, "PORT")
            .or_else(|| env::var("PORT").ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);

        let debug_mode = Self::get_value(map, "DEBUG")
            .or_else(|| env::var("DEBUG").ok())
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        let api_keys = Self::get_value(map, "API_KEYS")
            .or_else(|| env::var("API_KEYS").ok())
            .map(|s| s.split(',').map(|key| key.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);

        Ok(Config {
            database_url,
            port,
            debug_mode,
            api_keys,
        })
    }

    fn get_value(map: &HashMap<String, String>, key: &str) -> Option<String> {
        map.get(key).map(|s| s.to_string())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://prod:5432/db").unwrap();
        writeln!(temp_file, "PORT=9090").unwrap();
        writeln!(temp_file, "DEBUG=true").unwrap();
        writeln!(temp_file, "API_KEYS=key1,key2,key3").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://prod:5432/db");
        assert_eq!(config.port, 9090);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.api_keys, vec!["key1", "key2", "key3"]);
    }

    #[test]
    fn test_config_with_env_fallback() {
        env::set_var("DATABASE_URL", "postgres://env:5432/db");
        
        let config = Config::from_map(&HashMap::new()).unwrap();
        assert_eq!(config.database_url, "postgres://env:5432/db");
        assert_eq!(config.port, 8080);
        assert_eq!(config.debug_mode, false);
        
        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            port: 0,
            debug_mode: false,
            api_keys: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&"DATABASE_URL cannot be empty".to_string()));
        assert!(errors.contains(&"PORT must be greater than 0".to_string()));
    }
}