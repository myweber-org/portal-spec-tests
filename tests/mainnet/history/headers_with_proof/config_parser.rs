use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub pool_timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 10,
                pool_timeout_seconds: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "logs/app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, Box<dyn std::error::Error>> {
    if !path.as_ref().exists() {
        let default_config = AppConfig::default();
        let toml_content = toml::to_string_pretty(&default_config)?;
        fs::write(&path, toml_content)?;
        return Ok(default_config);
    }

    let content = fs::read_to_string(&path)?;
    let config: AppConfig = toml::from_str(&content)?;
    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be zero".to_string());
    }

    if config.database.max_connections == 0 {
        return Err("Database max connections cannot be zero".to_string());
    }

    if config.logging.max_file_size_mb == 0 {
        return Err("Log file max size cannot be zero".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_load_missing_config_creates_default() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        std::fs::remove_file(path).unwrap();

        let result = load_config(path);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_validation_rejects_invalid_config() {
        let mut config = AppConfig::default();
        config.server.port = 0;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port cannot be zero"));
    }
}