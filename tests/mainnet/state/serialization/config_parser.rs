use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
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
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
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
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "super-secret-key");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET_KEY=$APP_SECRET").unwrap();
        writeln!(file, "NON_EXISTENT=$UNKNOWN_VAR").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET_KEY").unwrap(), "super-secret-key");
        assert_eq!(config.get("NON_EXISTENT").unwrap(), "$UNKNOWN_VAR");
    }
}use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let mut processed_value = value.trim().to_string();
                
                for capture in var_pattern.captures_iter(&processed_value) {
                    if let Some(var_name) = capture.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            processed_value = processed_value.replace(&capture[0], &env_value);
                        }
                    }
                }
                
                self.values.insert(key, processed_value);
            }
        }
        
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = r#"
            server_host=localhost
            server_port=8080
            debug_mode=true
        "#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("server_port"), Some(&"8080".to_string()));
        assert_eq!(parser.get("debug_mode"), Some(&"true".to_string()));
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "3000");
        
        let mut parser = ConfigParser::new();
        let config = r#"
            host=127.0.0.1
            port=${APP_PORT}
            url=http://${host}:${APP_PORT}/api
        "#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("port"), Some(&"3000".to_string()));
        assert_eq!(parser.get("url"), Some(&"http://127.0.0.1:3000/api".to_string()));
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub numeric_values: HashMap<String, f64>,
    pub flags: HashMap<String, bool>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            numeric_values: HashMap::new(),
            flags: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        let mut current_section = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len()-1].to_string();
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config format at line {}", line_num + 1));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            match current_section.as_str() {
                "settings" => {
                    config.settings.insert(key, value);
                }
                "numeric" => {
                    let num_value = value.parse::<f64>()
                        .map_err(|_| format!("Invalid numeric value at line {}", line_num + 1))?;
                    config.numeric_values.insert(key, num_value);
                }
                "flags" => {
                    let flag_value = match value.to_lowercase().as_str() {
                        "true" | "yes" | "1" => true,
                        "false" | "no" | "0" => false,
                        _ => return Err(format!("Invalid boolean value at line {}", line_num + 1)),
                    };
                    config.flags.insert(key, flag_value);
                }
                _ => {
                    config.settings.insert(key, value);
                }
            }
        }

        Ok(config)
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_numeric(&self, key: &str) -> Option<f64> {
        self.numeric_values.get(key).copied()
    }

    pub fn get_flag(&self, key: &str) -> Option<bool> {
        self.flags.get(key).copied()
    }

    pub fn get_setting_with_default(&self, key: &str, default: &str) -> String {
        self.settings.get(key)
            .map(|s| s.clone())
            .unwrap_or_else(|| default.to_string())
    }

    pub fn get_numeric_with_default(&self, key: &str, default: f64) -> f64 {
        self.numeric_values.get(key)
            .copied()
            .unwrap_or(default)
    }

    pub fn get_flag_with_default(&self, key: &str, default: bool) -> bool {
        self.flags.get(key)
            .copied()
            .unwrap_or(default)
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
# Sample configuration
[settings]
app_name = MyApplication
log_level = INFO

[numeric]
timeout = 30.5
retry_count = 3

[flags]
enable_cache = true
debug_mode = false
"#;
        write!(temp_file, "{}", config_content).unwrap();
        
        let config = Config::load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.get_setting("app_name"), Some(&"MyApplication".to_string()));
        assert_eq!(config.get_numeric("timeout"), Some(30.5));
        assert_eq!(config.get_flag("enable_cache"), Some(true));
        assert_eq!(config.get_flag("debug_mode"), Some(false));
        
        assert_eq!(config.get_setting_with_default("missing", "default"), "default");
        assert_eq!(config.get_numeric_with_default("missing", 42.0), 42.0);
        assert_eq!(config.get_flag_with_default("missing", true), true);
    }

    #[test]
    fn test_invalid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = "invalid_line_without_equals";
        write!(temp_file, "{}", config_content).unwrap();
        
        let result = Config::load_from_file(temp_file.path());
        assert!(result.is_err());
    }
}use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    pub fn parse_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let env_var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                value = value.trim();

                let processed_value = env_var_pattern.replace_all(value, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
                });

                self.values.insert(key, processed_value.to_string());
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let content = "HOST=localhost\nPORT=8080\nDEBUG=true";
        parser.parse_content(content).unwrap();

        assert_eq!(parser.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(parser.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("DB_HOST", "postgresql");
        let mut parser = ConfigParser::new();
        let content = "DATABASE_URL=jdbc:postgresql://${DB_HOST}:5432/mydb";
        parser.parse_content(content).unwrap();

        let expected = "jdbc:postgresql://postgresql:5432/mydb";
        assert_eq!(parser.get("DATABASE_URL"), Some(&expected.to_string()));
    }

    #[test]
    fn test_file_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let mut parser = ConfigParser::new();
        parser.parse_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(parser.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(parser.get("TIMEOUT"), Some(&"30".to_string()));
        assert!(!parser.contains_key("COMMENT"));
    }
}