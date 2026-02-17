use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
    pub features: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.host.is_empty() {
            return Err("Host cannot be empty");
        }
        if self.port == 0 {
            return Err("Port must be greater than 0");
        }
        Ok(())
    }
}

pub fn parse_json<T>(json_str: &str) -> Result<T, Box<dyn Error>>
where
    T: for<'a> Deserialize<'a>,
{
    let result: T = serde_json::from_str(json_str)?;
    Ok(result)
}

pub fn to_json<T>(value: &T) -> Result<String, Box<dyn Error>>
where
    T: Serialize,
{
    let json = serde_json::to_string_pretty(value)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let json_data = r#"
        {
            "host": "localhost",
            "port": 8080,
            "debug": true,
            "features": ["auth", "logging"]
        }
        "#;

        let config: Config = parse_json(json_data).unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.debug, true);
        assert_eq!(config.features.len(), 2);
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            host: String::new(),
            port: 0,
            debug: false,
            features: vec![],
        };

        assert!(config.validate().is_err());
    }
}