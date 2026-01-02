use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub thresholds: HashMap<String, f64>,
    pub enabled_features: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        let mut thresholds = HashMap::new();
        let mut enabled_features = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').map(|s| s.trim()).collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0];
            let value = parts[1];

            if key.starts_with("threshold.") {
                let threshold_key = key.trim_start_matches("threshold.").to_string();
                let threshold_value: f64 = value
                    .parse()
                    .map_err(|_| format!("Invalid threshold value: {}", value))?;
                thresholds.insert(threshold_key, threshold_value);
            } else if key == "enabled_features" {
                enabled_features = value.split(',').map(|s| s.trim().to_string()).collect();
            } else {
                settings.insert(key.to_string(), value.to_string());
            }
        }

        Ok(Config {
            settings,
            thresholds,
            enabled_features,
        })
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_threshold(&self, key: &str) -> Option<&f64> {
        self.thresholds.get(key)
    }

    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.enabled_features.iter().any(|f| f == feature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_host=localhost
            server_port=8080
            threshold.cpu_usage=75.5
            threshold.memory_usage=90.0
            enabled_features=logging,monitoring,caching
        "#;
        write!(temp_file, "{}", config_content).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.get_setting("server_host"), Some(&"localhost".to_string()));
        assert_eq!(config.get_setting("server_port"), Some(&"8080".to_string()));
        assert_eq!(config.get_threshold("cpu_usage"), Some(&75.5));
        assert_eq!(config.get_threshold("memory_usage"), Some(&90.0));
        assert!(config.is_feature_enabled("logging"));
        assert!(config.is_feature_enabled("monitoring"));
        assert!(!config.is_feature_enabled("debug"));
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut values = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }
        
        Ok(Config { values })
    }
    
    fn process_value(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let env_var = &value[2..value.len() - 1];
            env::var(env_var).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_HOST=localhost").unwrap();
        writeln!(file, "DATABASE_PORT=5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NON_EXISTENT"), None);
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "env_secret_value");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET=${APP_SECRET}").unwrap();
        writeln!(file, "NORMAL=plain_value").unwrap();
        writeln!(file, "MISSING=${NON_EXISTENT_VAR}").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET"), Some(&"env_secret_value".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"plain_value".to_string()));
        assert_eq!(config.get("MISSING"), Some(&"${NON_EXISTENT_VAR}".to_string()));
    }
    
    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut values = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::resolve_value(&raw_value);

            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn resolve_value(raw: &str) -> String {
        if raw.starts_with("${") && raw.ends_with('}') {
            let var_name = &raw[2..raw.len() - 1];
            env::var(var_name).unwrap_or_else(|_| raw.to_string())
        } else {
            raw.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${DB_PASSWORD}").unwrap();
        writeln!(file, "HOST=${UNDEFINED_VAR}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("HOST"), Some(&"${UNDEFINED_VAR}".to_string()));
    }
}