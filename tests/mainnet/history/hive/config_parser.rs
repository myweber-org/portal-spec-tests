use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_tls: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: String::from("127.0.0.1"),
            port: 8080,
            max_connections: 100,
            enable_tls: false,
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: ServerConfig = toml::from_str(&config_str)?;
    
    validate_config(&config)?;
    Ok(config)
}

pub fn load_config_with_defaults<P: AsRef<Path>>(path: P) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    match load_config(path) {
        Ok(config) => Ok(config),
        Err(_) => {
            println!("Using default configuration");
            Ok(ServerConfig::default())
        }
    }
}

fn validate_config(config: &ServerConfig) -> Result<(), String> {
    if config.port == 0 {
        return Err("Port cannot be 0".to_string());
    }
    
    if config.max_connections == 0 {
        return Err("Max connections must be greater than 0".to_string());
    }
    
    if config.host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert!(!config.enable_tls);
    }

    #[test]
    fn test_load_valid_config() {
        let config_str = r#"
            host = "0.0.0.0"
            port = 9000
            max_connections = 500
            enable_tls = true
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_str).unwrap();
        
        let config = load_config(temp_file.path()).unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9000);
        assert_eq!(config.max_connections, 500);
        assert!(config.enable_tls);
    }

    #[test]
    fn test_validate_invalid_port() {
        let config = ServerConfig {
            host: "localhost".to_string(),
            port: 0,
            max_connections: 100,
            enable_tls: false,
        };
        
        assert!(validate_config(&config).is_err());
    }
}