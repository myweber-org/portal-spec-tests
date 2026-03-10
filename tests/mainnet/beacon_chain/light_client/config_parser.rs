use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line format: {}", line));
            }
            
            let key = parts[0].trim().to_string();
            let mut value = parts[1].trim().to_string();
            
            for cap in var_pattern.captures_iter(&value) {
                if let Some(var_name) = cap.get(1) {
                    if let Ok(env_value) = env::var(var_name.as_str()) {
                        value = value.replace(&cap[0], &env_value);
                    }
                }
            }
            
            self.values.insert(key, value);
        }
        
        Ok(())
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
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = "server_host = localhost\nserver_port = 8080";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("server_port"), Some(&"8080".to_string()));
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "3000");
        
        let mut parser = ConfigParser::new();
        let config = "port = ${APP_PORT}\nhost = 127.0.0.1";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("port"), Some(&"3000".to_string()));
    }
}