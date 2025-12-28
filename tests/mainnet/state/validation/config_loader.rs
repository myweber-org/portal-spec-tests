use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub max_connections: u32,
}

impl AppConfig {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(file_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;

        config.apply_environment_overrides();
        Ok(config)
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server_host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server_port = port_num;
            }
        }
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
        if let Ok(max_conn) = env::var("MAX_CONNECTIONS") {
            if let Ok(max_conn_num) = max_conn.parse() {
                self.max_connections = max_conn_num;
            }
        }
    }

    pub fn default() -> Self {
        AppConfig {
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            database_url: "postgresql://localhost:5432/appdb".to_string(),
            log_level: "info".to_string(),
            max_connections: 10,
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
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_host = "localhost"
            server_port = 3000
            database_url = "postgresql://test:5432/db"
            log_level = "debug"
            max_connections = 20
        "#;
        write!(temp_file, "{}", config_content).unwrap();

        let config = AppConfig::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.server_host, "localhost");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.database_url, "postgresql://test:5432/db");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.max_connections, 20);
    }

    #[test]
    fn test_environment_override() {
        env::set_var("SERVER_HOST", "0.0.0.0");
        env::set_var("LOG_LEVEL", "trace");

        let mut config = AppConfig::default();
        config.apply_environment_overrides();

        assert_eq!(config.server_host, "0.0.0.0");
        assert_eq!(config.log_level, "trace");

        env::remove_var("SERVER_HOST");
        env::remove_var("LOG_LEVEL");
    }
}