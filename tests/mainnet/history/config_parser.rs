
use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Debug)]
pub struct Config {
    sections: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            sections: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> io::Result<Self> {
        let mut config = Config::new();
        let mut current_section = String::from("default");

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                config.sections.entry(current_section.clone()).or_insert_with(HashMap::new);
            } else if let Some(equal_pos) = line.find('=') {
                let key = line[..equal_pos].trim().to_string();
                let value = line[equal_pos + 1..].trim().to_string();
                config.sections
                    .entry(current_section.clone())
                    .or_insert_with(HashMap::new)
                    .insert(key, value);
            }
        }

        Ok(config)
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section)?.get(key)
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        self.sections
            .entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
    }

    pub fn sections(&self) -> Vec<&String> {
        self.sections.keys().collect()
    }

    pub fn keys_in_section(&self, section: &str) -> Option<Vec<&String>> {
        Some(self.sections.get(section)?.keys().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
# Sample config
[server]
host = localhost
port = 8080

[database]
url = postgresql://localhost/mydb
user = admin
"#;

        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("server", "host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("server", "port"), Some(&"8080".to_string()));
        assert_eq!(config.get("database", "url"), Some(&"postgresql://localhost/mydb".to_string()));
        assert_eq!(config.get("database", "user"), Some(&"admin".to_string()));
        assert_eq!(config.get("nonexistent", "key"), None);
    }

    #[test]
    fn test_modification() {
        let mut config = Config::new();
        config.set("general", "timeout", "30");
        config.set("general", "retries", "3");

        assert_eq!(config.get("general", "timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("general", "retries"), Some(&"3".to_string()));
    }
}