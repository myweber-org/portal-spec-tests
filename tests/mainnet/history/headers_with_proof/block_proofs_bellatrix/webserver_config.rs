use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub max_connections: usize,
    pub static_dir: PathBuf,
    pub enable_tls: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid port number: {0}")]
    InvalidPort(u16),
    #[error("Max connections must be positive, got {0}")]
    InvalidMaxConnections(usize),
    #[error("Static directory does not exist: {0}")]
    MissingStaticDir(String),
    #[error("TLS enabled but certificate path not provided")]
    MissingCertPath,
    #[error("TLS enabled but key path not provided")]
    MissingKeyPath,
    #[error("Certificate file not found: {0}")]
    CertFileNotFound(String),
    #[error("Key file not found: {0}")]
    KeyFileNotFound(String),
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.port == 0 {
            return Err(ConfigError::InvalidPort(self.port));
        }

        if self.max_connections == 0 {
            return Err(ConfigError::InvalidMaxConnections(self.max_connections));
        }

        if !self.static_dir.exists() {
            return Err(ConfigError::MissingStaticDir(
                self.static_dir.to_string_lossy().into_owned(),
            ));
        }

        if self.enable_tls {
            match &self.cert_path {
                Some(path) => {
                    if !path.exists() {
                        return Err(ConfigError::CertFileNotFound(
                            path.to_string_lossy().into_owned(),
                        ));
                    }
                }
                None => return Err(ConfigError::MissingCertPath),
            }

            match &self.key_path {
                Some(path) => {
                    if !path.exists() {
                        return Err(ConfigError::KeyFileNotFound(
                            path.to_string_lossy().into_owned(),
                        ));
                    }
                }
                None => return Err(ConfigError::MissingKeyPath),
            }
        }

        Ok(())
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".parse().unwrap(),
            port: 8080,
            max_connections: 100,
            static_dir: PathBuf::from("./static"),
            enable_tls: false,
            cert_path: None,
            key_path: None,
        }
    }
}