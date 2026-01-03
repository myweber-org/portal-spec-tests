
use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut values = HashMap::new();
        let var_regex = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                value = value.trim();

                let mut processed_value = value.to_string();
                for cap in var_regex.captures_iter(value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            processed_value = processed_value.replace(&cap[0], &env_value);
                        }
                    }
                }

                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
            server_host=localhost
            server_port=8080
            debug_mode=true
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("server_host").unwrap(), "localhost");
        assert_eq!(config.get("server_port").unwrap(), "8080");
        assert_eq!(config.get("debug_mode").unwrap(), "true");
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let content = r#"
            database_url=postgres://user:${DB_PASSWORD}@localhost/db
            api_key=${NONEXISTENT_VAR:-default_key}
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("database_url").unwrap(), "postgres://user:secret123@localhost/db");
        assert_eq!(config.get_or_default("api_key", "fallback"), "default_key");
    }

    #[test]
    fn test_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "app_name=TestApp").unwrap();
        writeln!(temp_file, "version=1.0.0").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("app_name").unwrap(), "TestApp");
        assert_eq!(config.get("version").unwrap(), "1.0.0");
    }

    #[test]
    fn test_skip_comments_and_blank_lines() {
        let content = r#"
            # This is a comment
            key1=value1
            
            # Another comment
            key2=value2
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.values.len(), 2);
        assert!(config.contains_key("key1"));
        assert!(config.contains_key("key2"));
    }
}