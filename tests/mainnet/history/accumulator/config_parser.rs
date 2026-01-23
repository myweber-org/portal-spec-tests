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
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug_mode: bool,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;

        config.apply_environment_overrides();
        config.validate()?;

        Ok(config)
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(debug_env) = env::var("APP_DEBUG_MODE") {
            if let Ok(debug_bool) = debug_env.parse::<bool>() {
                self.debug_mode = debug_bool;
            }
        }

        if let Ok(db_host) = env::var("DATABASE_HOST") {
            self.database.host = db_host;
        }

        if let Ok(server_port) = env::var("SERVER_PORT") {
            if let Ok(port) = server_port.parse::<u16>() {
                self.server.port = port;
            }
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

        Ok(())
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database_name
        )
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let config_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            database_name = "mydb"

            [server]
            address = "0.0.0.0"
            port = 8080
            max_connections = 100

            debug_mode = false
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert!(!config.debug_mode);
    }

    #[test]
    fn test_environment_overrides() {
        env::set_var("APP_DEBUG_MODE", "true");
        env::set_var("DATABASE_HOST", "prod-db.example.com");

        let config_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            database_name = "mydb"

            [server]
            address = "0.0.0.0"
            port = 8080
            max_connections = 100

            debug_mode = false
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "prod-db.example.com");
        assert!(config.debug_mode);

        env::remove_var("APP_DEBUG_MODE");
        env::remove_var("DATABASE_HOST");
    }

    #[test]
    fn test_validation() {
        let invalid_config = r#"
            [database]
            host = "localhost"
            port = 0
            username = "admin"
            password = "secret"
            database_name = "mydb"

            [server]
            address = "0.0.0.0"
            port = 8080
            max_connections = 100

            debug_mode = false
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), invalid_config).unwrap();

        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }
}