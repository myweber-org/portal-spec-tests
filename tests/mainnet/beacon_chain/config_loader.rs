use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let file_contents = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
        
        let mut config: HashMap<String, toml::Value> = toml::from_str(&file_contents)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        Self::apply_env_overrides(&mut config);
        
        Ok(Config {
            database_url: Self::get_string(&config, "database_url")?,
            port: Self::get_u16(&config, "port")?,
            debug_mode: Self::get_bool(&config, "debug_mode")?,
            api_keys: Self::get_api_keys(&config)?,
        })
    }
    
    fn apply_env_overrides(config: &mut HashMap<String, toml::Value>) {
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.insert("database_url".to_string(), toml::Value::String(db_url));
        }
        
        if let Ok(port) = env::var("PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.insert("port".to_string(), toml::Value::Integer(port_num as i64));
            }
        }
        
        if let Ok(debug) = env::var("DEBUG_MODE") {
            let debug_bool = debug.to_lowercase() == "true" || debug == "1";
            config.insert("debug_mode".to_string(), toml::Value::Boolean(debug_bool));
        }
    }
    
    fn get_string(config: &HashMap<String, toml::Value>, key: &str) -> Result<String, String> {
        config.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Missing or invalid string value for key: {}", key))
    }
    
    fn get_u16(config: &HashMap<String, toml::Value>, key: &str) -> Result<u16, String> {
        config.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| if i >= 0 && i <= u16::MAX as i64 { Some(i as u16) } else { None })
            .ok_or_else(|| format!("Missing or invalid u16 value for key: {}", key))
    }
    
    fn get_bool(config: &HashMap<String, toml::Value>, key: &str) -> Result<bool, String> {
        config.get(key)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| format!("Missing or invalid boolean value for key: {}", key))
    }
    
    fn get_api_keys(config: &HashMap<String, toml::Value>) -> Result<HashMap<String, String>, String> {
        let mut api_keys = HashMap::new();
        
        if let Some(keys_table) = config.get("api_keys").and_then(|v| v.as_table()) {
            for (service, value) in keys_table {
                if let Some(key_str) = value.as_str() {
                    api_keys.insert(service.clone(), key_str.to_string());
                }
            }
        }
        
        Ok(api_keys)
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let processed_value = Self::substitute_env_vars(value.trim());
                settings.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { settings })
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        for (key, env_value) in env::vars() {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, &env_value);
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let content = fs::read_to_string("config.toml")
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut table = toml::from_str::<toml::Value>(&content)
            .map_err(|e| format!("Invalid TOML: {}", e))?
            .as_table()
            .ok_or("Config must be a TOML table")?
            .clone();

        Self::substitute_env_vars(&mut table);

        let config: Config = toml::from_str(&toml::to_string(&table).unwrap())
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        Ok(config)
    }

    fn substitute_env_vars(table: &mut toml::value::Table) {
        let mut replacements = HashMap::new();

        for (key, value) in table.iter() {
            if let Some(s) = value.as_str() {
                if s.starts_with("${") && s.ends_with('}') {
                    let var_name = &s[2..s.len() - 1];
                    if let Ok(env_value) = env::var(var_name) {
                        replacements.insert(key.clone(), toml::Value::String(env_value));
                    }
                }
            } else if let Some(arr) = value.as_array() {
                let mut new_arr = Vec::new();
                for item in arr {
                    if let Some(s) = item.as_str() {
                        if s.starts_with("${") && s.ends_with('}') {
                            let var_name = &s[2..s.len() - 1];
                            if let Ok(env_value) = env::var(var_name) {
                                new_arr.push(toml::Value::String(env_value));
                                continue;
                            }
                        }
                    }
                    new_arr.push(item.clone());
                }
                replacements.insert(key.clone(), toml::Value::Array(new_arr));
            } else if let Some(sub_table) = value.as_table() {
                let mut sub_table_clone = sub_table.clone();
                Self::substitute_env_vars(&mut sub_table_clone);
                replacements.insert(key.clone(), toml::Value::Table(sub_table_clone));
            }
        }

        for (key, value) in replacements {
            table.insert(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        env::set_var("API_SECRET", "supersecret");

        let toml_content = r#"
            database_url = "${DB_HOST}:5432"
            api_key = "${API_SECRET}"
            debug_mode = true
            port = 8080
        "#;

        fs::write("test_config.toml", toml_content).unwrap();
        
        let config = Config::load().unwrap();
        assert_eq!(config.database_url, "localhost:5432");
        assert_eq!(config.api_key, "supersecret");
        
        fs::remove_file("test_config.toml").unwrap();
    }
}