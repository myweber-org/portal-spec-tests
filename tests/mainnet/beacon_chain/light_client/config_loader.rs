use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let mut config: AppConfig = toml::from_str(&content)?;

        if let Ok(port) = env::var("APP_PORT") {
            config.server_port = port.parse()?;
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level;
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero");
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty");
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err("Invalid log level specified");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        write!(temp_file, "{}", config_content).unwrap();

        let config = AppConfig::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost/mydb");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_environment_override() {
        env::set_var("APP_PORT", "9090");
        env::set_var("DATABASE_URL", "postgres://prod/db");

        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        write!(temp_file, "{}", config_content).unwrap();

        let config = AppConfig::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.database_url, "postgres://prod/db");

        env::remove_var("APP_PORT");
        env::remove_var("DATABASE_URL");
    }
}