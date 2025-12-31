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
        let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.*?)\s*$").unwrap();
        let var_re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();

                // Replace environment variables
                for var_caps in var_re.captures_iter(&value) {
                    let var_name = &var_caps[1];
                    if let Ok(var_value) = env::var(var_name) {
                        value = value.replace(&var_caps[0], &var_value);
                    }
                }

                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = r#"
            database_host = localhost
            database_port = 5432
            # This is a comment
            api_key = secret_value
        "#;

        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("database_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("database_port"), Some(&"5432".to_string()));
        assert_eq!(parser.get("api_key"), Some(&"secret_value".to_string()));
        assert_eq!(parser.get("nonexistent"), None);
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("APP_MODE", "production");
        
        let mut parser = ConfigParser::new();
        let config = r#"
            mode = ${APP_MODE}
            path = /home/${USER}/data
        "#;

        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("mode"), Some(&"production".to_string()));
    }

    #[test]
    fn test_invalid_syntax() {
        let mut parser = ConfigParser::new();
        let config = r#"
            valid_key = valid_value
            invalid line without equals
            another_valid = value
        "#;

        assert!(parser.load_from_str(config).is_err());
    }
}