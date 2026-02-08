
use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, PartialEq)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let parsed: toml::Value = content.parse()
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;
        
        let database_table = parsed.get("database")
            .ok_or("Missing 'database' section")?
            .as_table()
            .ok_or("'database' section must be a table")?;
        
        let server_table = parsed.get("server")
            .ok_or("Missing 'server' section")?
            .as_table()
            .ok_or("'server' section must be a table")?;
        
        let features_table = parsed.get("features")
            .map(|v| v.as_table().unwrap_or(&toml::value::Table::new()))
            .unwrap_or(&toml::value::Table::new());
        
        let database = DatabaseConfig {
            host: database_table.get("host")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid database.host")?
                .to_string(),
            port: database_table.get("port")
                .and_then(|v| v.as_integer())
                .ok_or("Missing or invalid database.port")?
                .try_into()
                .map_err(|_| "database.port out of range")?,
            username: database_table.get("username")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid database.username")?
                .to_string(),
            password: database_table.get("password")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid database.password")?
                .to_string(),
            name: database_table.get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid database.name")?
                .to_string(),
        };
        
        let server = ServerConfig {
            host: server_table.get("host")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid server.host")?
                .to_string(),
            port: server_table.get("port")
                .and_then(|v| v.as_integer())
                .ok_or("Missing or invalid server.port")?
                .try_into()
                .map_err(|_| "server.port out of range")?,
            tls_enabled: server_table.get("tls_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        };
        
        let mut features = HashMap::new();
        for (key, value) in features_table {
            if let Some(bool_val) = value.as_bool() {
                features.insert(key.clone(), bool_val);
            }
        }
        
        Ok(Config {
            database,
            server,
            features,
        })
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.database.port == 0 {
            return Err("Database port cannot be 0".to_string());
        }
        
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        
        if self.server.tls_enabled && self.server.port == 80 {
            return Err("TLS should not be enabled on port 80".to_string());
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
    fn test_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            name = "app_db"
            
            [server]
            host = "0.0.0.0"
            port = 8080
            tls_enabled = true
            
            [features]
            caching = true
            logging = false
        "#;
        
        write!(temp_file, "{}", config_content).unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.tls_enabled, true);
        assert_eq!(config.features.get("caching"), Some(&true));
        assert_eq!(config.features.get("logging"), Some(&false));
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_missing_section() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            [database]
            host = "localhost"
            port = 5432
        "#;
        
        write!(temp_file, "{}", config_content).unwrap();
        
        let result = Config::from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'server' section"));
    }
    
    #[test]
    fn test_validation_failure() {
        let config = Config {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 0,
                username: "admin".to_string(),
                password: "secret".to_string(),
                name: "db".to_string(),
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 80,
                tls_enabled: true,
            },
            features: HashMap::new(),
        };
        
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Database port cannot be 0") || 
                err.contains("TLS should not be enabled on port 80"));
    }
}