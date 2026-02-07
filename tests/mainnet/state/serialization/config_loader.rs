use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub enable_cache: bool,
}

impl AppConfig {
    pub fn from_file(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;

        config.apply_environment_overrides();
        config.validate()?;

        Ok(config)
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(db_host) = env::var("DB_HOST") {
            self.database.host = db_host;
        }
        if let Ok(db_port) = env::var("DB_PORT") {
            if let Ok(port) = db_port.parse() {
                self.database.port = port;
            }
        }
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database.port == 0 {
            return Err("Database port cannot be zero".to_string());
        }
        if self.server.max_connections == 0 {
            return Err("Max connections must be greater than zero".to_string());
        }
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        Ok(())
    }
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_path = env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config.toml".to_string());

    if !Path::new(&config_path).exists() {
        return Err(format!("Configuration file not found: {}", config_path).into());
    }

    AppConfig::from_file(&config_path)
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut parsed = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = Self::resolve_value(value.trim());
                parsed.insert(key, value);
            }
        }

        Ok(Config {
            database_url: parsed
                .get("DATABASE_URL")
                .ok_or("Missing DATABASE_URL")?
                .clone(),
            api_key: parsed.get("API_KEY").ok_or("Missing API_KEY")?.clone(),
            debug_mode: parsed
                .get("DEBUG")
                .map(|v| v == "true")
                .unwrap_or(false),
            port: parsed
                .get("PORT")
                .map(|v| v.parse().unwrap_or(8080))
                .unwrap_or(8080),
        })
    }

    fn resolve_value(value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "API_KEY=$SECRET_KEY").unwrap();
        writeln!(file, "DEBUG=true").unwrap();
        writeln!(file, "PORT=3000").unwrap();

        env::set_var("SECRET_KEY", "abc123");

        let config = Config::load(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.api_key, "abc123");
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.port, 3000);
    }
}