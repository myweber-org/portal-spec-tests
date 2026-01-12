use std::collections::HashMap;
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
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = line.split_once('=') {
                let key = key.trim().to_string();
                
                for cap in var_pattern.captures_iter(&value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            value = value.replace(&cap[0], &env_value);
                        }
                    }
                }
                
                self.values.insert(key, value.trim().to_string());
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let content = "DATABASE_URL=postgres://localhost:5432/mydb\nAPI_KEY=secret123\n";
        
        parser.parse_content(content).unwrap();
        
        assert_eq!(parser.get("DATABASE_URL").unwrap(), "postgres://localhost:5432/mydb");
        assert_eq!(parser.get("API_KEY").unwrap(), "secret123");
        assert!(parser.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "8080");
        
        let mut parser = ConfigParser::new();
        let content = "PORT=${APP_PORT}\nHOST=localhost\n";
        
        parser.parse_content(content).unwrap();
        
        assert_eq!(parser.get("PORT").unwrap(), "8080");
        assert_eq!(parser.get("HOST").unwrap(), "localhost");
    }

    #[test]
    fn test_file_parsing() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.cfg");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "DEBUG=true\nLOG_LEVEL=info\n").unwrap();
        
        let mut parser = ConfigParser::new();
        parser.parse_file(file_path.to_str().unwrap()).unwrap();
        
        assert_eq!(parser.get("DEBUG").unwrap(), "true");
        assert_eq!(parser.get("LOG_LEVEL").unwrap(), "info");
    }
}