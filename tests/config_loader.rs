use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = trimmed.split_once('=') {
                value = value.trim();
                let expanded = Self::expand_env_vars(value);
                settings.insert(key.trim().to_string(), expanded);
            }
        }

        Ok(Config { settings })
    }

    fn expand_env_vars(input: &str) -> String {
        let mut result = input.to_string();
        for (key, value) in env::vars() {
            let placeholder = format!("${}", key);
            result = result.replace(&placeholder, &value);
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}