use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&config_str)?;
        
        config.apply_environment_overrides();
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("APP_SERVER_HOST") {
            self.server.host = host;
        }
        
        if let Ok(port) = env::var("APP_SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server.port = port_num;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.logging.level = log_level;
        }
    }
    
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".into());
        }
        
        if self.database.max_connections == 0 {
            return Err("Database max connections must be greater than zero".into());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.to_lowercase().as_str()) {
            return Err(format!("Invalid log level: {}", self.logging.level).into());
        }
        
        Ok(())
    }
    
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_config_parsing() {
        let toml_content = r#"
            [server]
            host = "127.0.0.1"
            port = 8080
            tls_enabled = false
            
            [database]
            url = "postgres://localhost/mydb"
            max_connections = 10
            timeout_seconds = 30
            
            [logging]
            level = "info"
            file_path = "/var/log/app.log"
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, toml_content.as_bytes()).unwrap();
        
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.logging.level, "info");
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("APP_SERVER_PORT", "9090");
        env::set_var("LOG_LEVEL", "debug");
        
        let toml_content = r#"
            [server]
            host = "localhost"
            port = 8080
            tls_enabled = false
            
            [database]
            url = "postgres://localhost/test"
            max_connections = 5
            timeout_seconds = 10
            
            [logging]
            level = "info"
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, toml_content.as_bytes()).unwrap();
        
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.logging.level, "debug");
        
        env::remove_var("APP_SERVER_PORT");
        env::remove_var("LOG_LEVEL");
    }
    
    #[test]
    fn test_validation_failure() {
        let toml_content = r#"
            [server]
            host = "localhost"
            port = 0
            tls_enabled = false
            
            [database]
            url = "postgres://localhost/test"
            max_connections = 5
            timeout_seconds = 10
            
            [logging]
            level = "info"
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, toml_content.as_bytes()).unwrap();
        
        let result = AppConfig::from_file(file.path());
        assert!(result.is_err());
    }
}