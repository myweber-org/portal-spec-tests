use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub debug_mode: bool,
    pub api_keys: Vec<String>,
    pub timeout_seconds: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                config_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    pub fn from_env() -> Result<Self, String> {
        let mut config_map = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                config_map.insert(key.trim_start_matches("APP_").to_string(), value);
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = map
            .get("DATABASE_URL")
            .map(|s| s.to_string())
            .ok_or("Missing DATABASE_URL")?;

        let server_port = map
            .get("SERVER_PORT")
            .map(|s| s.parse::<u16>())
            .transpose()
            .map_err(|e| format!("Invalid SERVER_PORT: {}", e))?
            .unwrap_or(8080);

        let debug_mode = map
            .get("DEBUG_MODE")
            .map(|s| s.parse::<bool>())
            .transpose()
            .map_err(|e| format!("Invalid DEBUG_MODE: {}", e))?
            .unwrap_or(false);

        let api_keys = map
            .get("API_KEYS")
            .map(|s| s.split(',').map(|key| key.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);

        let timeout_seconds = map
            .get("TIMEOUT_SECONDS")
            .map(|s| s.parse::<u64>())
            .transpose()
            .map_err(|e| format!("Invalid TIMEOUT_SECONDS: {}", e))?
            .unwrap_or(30);

        Ok(Config {
            database_url,
            server_port,
            debug_mode,
            api_keys,
            timeout_seconds,
        })
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }

        if self.server_port == 0 {
            errors.push("SERVER_PORT cannot be zero".to_string());
        }

        if self.timeout_seconds == 0 {
            errors.push("TIMEOUT_SECONDS cannot be zero".to_string());
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
    fn test_config_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(temp_file, "SERVER_PORT=3000").unwrap();
        writeln!(temp_file, "DEBUG_MODE=true").unwrap();
        writeln!(temp_file, "API_KEYS=key1,key2,key3").unwrap();
        writeln!(temp_file, "TIMEOUT_SECONDS=60").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.api_keys, vec!["key1", "key2", "key3"]);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            server_port: 0,
            debug_mode: false,
            api_keys: vec![],
            timeout_seconds: 0,
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}