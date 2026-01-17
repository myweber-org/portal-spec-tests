use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn default_config() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/mydb".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("app.log".to_string()),
                enable_console: true,
            },
        }
    }

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        env_vars.insert("SERVER_HOST".to_string(), self.server.host.clone());
        env_vars.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        env_vars.insert("DB_URL".to_string(), self.database.url.clone());
        env_vars.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        env_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default_config();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        config.to_file(path).unwrap();
        let loaded_config = AppConfig::from_file(path).unwrap();

        assert_eq!(config.server.host, loaded_config.server.host);
        assert_eq!(config.server.port, loaded_config.server.port);
        assert_eq!(config.database.url, loaded_config.database.url);
        assert_eq!(config.logging.level, loaded_config.logging.level);
    }

    #[test]
    fn test_env_vars_conversion() {
        let config = AppConfig::default_config();
        let env_vars = config.to_env_vars();

        assert_eq!(env_vars.get("SERVER_HOST"), Some(&"127.0.0.1".to_string()));
        assert_eq!(env_vars.get("SERVER_PORT"), Some(&"8080".to_string()));
        assert_eq!(env_vars.get("DB_URL"), Some(&"postgresql://localhost/mydb".to_string()));
        assert_eq!(env_vars.get("LOG_LEVEL"), Some(&"info".to_string()));
    }
}