use std::collections::HashMap;
use std::env;
use std::fs;

use toml::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub enable_compression: bool,
}

#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub enable_cache: bool,
    pub enable_logging: bool,
    pub debug_mode: bool,
    pub maintenance_mode: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let parsed: Value = contents
            .parse()
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;

        let mut config = Self::default();
        config.apply_toml(&parsed)?;
        config.apply_environment_overrides();

        Ok(config)
    }

    fn apply_toml(&mut self, value: &Value) -> Result<(), String> {
        if let Some(database) = value.get("database") {
            self.database.apply_toml(database)?;
        }

        if let Some(server) = value.get("server") {
            self.server.apply_toml(server)?;
        }

        if let Some(features) = value.get("features") {
            self.features.apply_toml(features)?;
        }

        Ok(())
    }

    fn apply_environment_overrides(&mut self) {
        self.database.apply_environment_overrides();
        self.server.apply_environment_overrides();
        self.features.apply_environment_overrides();
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            features: FeatureFlags::default(),
        }
    }
}

impl DatabaseConfig {
    fn apply_toml(&mut self, value: &Value) -> Result<(), String> {
        if let Some(host) = value.get("host").and_then(|v| v.as_str()) {
            self.host = host.to_string();
        }

        if let Some(port) = value.get("port").and_then(|v| v.as_integer()) {
            self.port = port as u16;
        }

        if let Some(username) = value.get("username").and_then(|v| v.as_str()) {
            self.username = username.to_string();
        }

        if let Some(password) = value.get("password").and_then(|v| v.as_str()) {
            self.password = password.to_string();
        }

        if let Some(db_name) = value.get("database_name").and_then(|v| v.as_str()) {
            self.database_name = db_name.to_string();
        }

        if let Some(pool_size) = value.get("pool_size").and_then(|v| v.as_integer()) {
            self.pool_size = pool_size as u32;
        }

        Ok(())
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("DB_HOST") {
            self.host = host;
        }

        if let Ok(port) = env::var("DB_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.port = port_num;
            }
        }

        if let Ok(username) = env::var("DB_USERNAME") {
            self.username = username;
        }

        if let Ok(password) = env::var("DB_PASSWORD") {
            self.password = password;
        }

        if let Ok(db_name) = env::var("DB_NAME") {
            self.database_name = db_name;
        }

        if let Ok(pool_size) = env::var("DB_POOL_SIZE") {
            if let Ok(size) = pool_size.parse::<u32>() {
                self.pool_size = size;
            }
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "".to_string(),
            database_name: "app_db".to_string(),
            pool_size: 10,
        }
    }
}

impl ServerConfig {
    fn apply_toml(&mut self, value: &Value) -> Result<(), String> {
        if let Some(host) = value.get("host").and_then(|v| v.as_str()) {
            self.host = host.to_string();
        }

        if let Some(port) = value.get("port").and_then(|v| v.as_integer()) {
            self.port = port as u16;
        }

        if let Some(max_conn) = value.get("max_connections").and_then(|v| v.as_integer()) {
            self.max_connections = max_conn as u32;
        }

        if let Some(timeout) = value.get("timeout_seconds").and_then(|v| v.as_integer()) {
            self.timeout_seconds = timeout as u64;
        }

        if let Some(enable_comp) = value.get("enable_compression").and_then(|v| v.as_bool()) {
            self.enable_compression = enable_comp;
        }

        Ok(())
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("SERVER_HOST") {
            self.host = host;
        }

        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.port = port_num;
            }
        }

        if let Ok(max_conn) = env::var("SERVER_MAX_CONNECTIONS") {
            if let Ok(max) = max_conn.parse::<u32>() {
                self.max_connections = max;
            }
        }

        if let Ok(timeout) = env::var("SERVER_TIMEOUT") {
            if let Ok(seconds) = timeout.parse::<u64>() {
                self.timeout_seconds = seconds;
            }
        }

        if let Ok(enable_comp) = env::var("SERVER_ENABLE_COMPRESSION") {
            self.enable_compression = enable_comp.to_lowercase() == "true";
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            max_connections: 100,
            timeout_seconds: 30,
            enable_compression: true,
        }
    }
}

impl FeatureFlags {
    fn apply_toml(&mut self, value: &Value) -> Result<(), String> {
        if let Some(enable_cache) = value.get("enable_cache").and_then(|v| v.as_bool()) {
            self.enable_cache = enable_cache;
        }

        if let Some(enable_logging) = value.get("enable_logging").and_then(|v| v.as_bool()) {
            self.enable_logging = enable_logging;
        }

        if let Some(debug_mode) = value.get("debug_mode").and_then(|v| v.as_bool()) {
            self.debug_mode = debug_mode;
        }

        if let Some(maintenance_mode) = value.get("maintenance_mode").and_then(|v| v.as_bool()) {
            self.maintenance_mode = maintenance_mode;
        }

        Ok(())
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(enable_cache) = env::var("FEATURE_ENABLE_CACHE") {
            self.enable_cache = enable_cache.to_lowercase() == "true";
        }

        if let Ok(enable_logging) = env::var("FEATURE_ENABLE_LOGGING") {
            self.enable_logging = enable_logging.to_lowercase() == "true";
        }

        if let Ok(debug_mode) = env::var("FEATURE_DEBUG_MODE") {
            self.debug_mode = debug_mode.to_lowercase() == "true";
        }

        if let Ok(maintenance_mode) = env::var("FEATURE_MAINTENANCE_MODE") {
            self.maintenance_mode = maintenance_mode.to_lowercase() == "true";
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_cache: true,
            enable_logging: true,
            debug_mode: false,
            maintenance_mode: false,
        }
    }
}

pub fn load_config() -> Result<Config, String> {
    let config_paths = vec![
        "config.toml",
        "config/config.toml",
        "/etc/app/config.toml",
    ];

    for path in config_paths {
        if let Ok(config) = Config::from_file(path) {
            return Ok(config);
        }
    }

    let mut config = Config::default();
    config.apply_environment_overrides();
    Ok(config)
}