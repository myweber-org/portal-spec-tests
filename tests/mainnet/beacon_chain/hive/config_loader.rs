use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config_map: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                config_map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Self::from_map(config_map)
    }

    fn from_map(mut map: HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = Self::get_value(&mut map, "DATABASE_URL")?;
        let server_port = Self::get_value(&mut map, "SERVER_PORT")?.parse()?;
        let log_level = Self::get_value(&mut map, "LOG_LEVEL")?;

        let mut features = HashMap::new();
        for (key, value) in map {
            if key.starts_with("FEATURE_") {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.parse::<bool>().unwrap_or(false);
                features.insert(feature_name, enabled);
            }
        }

        Ok(Config {
            database_url,
            server_port,
            log_level,
            features,
        })
    }

    fn get_value(map: &mut HashMap<String, String>, key: &str) -> Result<String, String> {
        if let Ok(env_value) = env::var(key) {
            return Ok(env_value);
        }

        map.remove(key)
            .ok_or_else(|| format!("Missing required configuration: {}", key))
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
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(temp_file, "SERVER_PORT=8080").unwrap();
        writeln!(temp_file, "LOG_LEVEL=info").unwrap();
        writeln!(temp_file, "FEATURE_CACHE=true").unwrap();
        writeln!(temp_file, "FEATURE_DEBUG=false").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.features.get("cache"), Some(&true));
        assert_eq!(config.features.get("debug"), Some(&false));
    }

    #[test]
    fn test_env_override() {
        env::set_var("SERVER_PORT", "9090");
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(temp_file, "SERVER_PORT=8080").unwrap();
        writeln!(temp_file, "LOG_LEVEL=debug").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.server_port, 9090);
        env::remove_var("SERVER_PORT");
    }
}