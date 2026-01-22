use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

pub struct JsonParser {
    input: String,
    pos: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.to_string(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        let c = self.input.chars().nth(self.pos).unwrap();
        match c {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", c)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with("null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip opening quote
        let start = self.pos;
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '"' {
                self.pos += 1;
                return Ok(JsonValue::String(result));
            } else if c == '\\' {
                self.pos += 1;
                if self.pos >= self.input.len() {
                    return Err("Unexpected end of string".to_string());
                }
                let escaped = self.input.chars().nth(self.pos).unwrap();
                match escaped {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
                }
                self.pos += 1;
            } else {
                result.push(c);
                self.pos += 1;
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '.' {
                if has_dot {
                    return Err("Invalid number format".to_string());
                }
                has_dot = true;
                self.pos += 1;
            } else if c.is_digit(10) {
                self.pos += 1;
            } else {
                break;
            }
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '['
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == ']' {
            self.pos += 1;
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                return Err("Unterminated array".to_string());
            }

            let c = self.input.chars().nth(self.pos).unwrap();
            if c == ']' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or ']', found: {}", c));
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '{'
        self.skip_whitespace();
        let mut object = HashMap::new();

        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos).unwrap() != ':' {
                return Err("Expected ':' after object key".to_string());
            }
            self.pos += 1;

            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err("Unterminated object".to_string());
            }

            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '}' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or '}}', found: {}", c));
            }
        }

        Ok(JsonValue::Object(object))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null");
        assert_eq!(parser.parse(), Ok(JsonValue::Null));
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(true)));

        let mut parser = JsonParser::new("false");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(false)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
        );
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));

        let mut parser = JsonParser::new("3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(3.14)));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]))
        );
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));
    }
}use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
    preferences: HashMap<String, Value>,
}

impl User {
    fn from_json_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| serde_json::Error::io(e))?;
        
        let user: User = serde_json::from_str(&content)?;
        
        if user.email.is_empty() || !user.email.contains('@') {
            return Err(serde_json::Error::custom("Invalid email format"));
        }
        
        Ok(user)
    }
    
    fn to_pretty_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
    }
    
    fn update_preference(&mut self, key: &str, value: Value) {
        self.preferences.insert(key.to_string(), value);
    }
}

fn validate_json_structure(json_str: &str) -> Result<()> {
    let value: Value = serde_json::from_str(json_str)?;
    
    if !value.is_object() {
        return Err(serde_json::Error::custom("Expected JSON object"));
    }
    
    let obj = value.as_object().unwrap();
    if !obj.contains_key("id") || !obj.contains_key("name") {
        return Err(serde_json::Error::custom("Missing required fields"));
    }
    
    Ok(())
}

fn parse_json_array(json_str: &str) -> Result<Vec<User>> {
    let users: Vec<User> = serde_json::from_str(json_str)?;
    
    for user in &users {
        if user.id == 0 {
            return Err(serde_json::Error::custom("Invalid user ID"));
        }
    }
    
    Ok(users)
}

fn main() -> Result<()> {
    let sample_json = r#"
        {
            "id": 42,
            "name": "John Doe",
            "email": "john@example.com",
            "active": true,
            "preferences": {
                "theme": "dark",
                "notifications": true
            }
        }
    "#;
    
    validate_json_structure(sample_json)?;
    
    let mut user: User = serde_json::from_str(sample_json)?;
    println!("Parsed user: {:?}", user);
    
    user.update_preference("language", Value::String("en-US".to_string()));
    
    let json_output = user.to_pretty_json()?;
    println!("Updated JSON:\n{}", json_output);
    
    let users_json = r#"
        [
            {
                "id": 1,
                "name": "Alice",
                "email": "alice@example.com",
                "active": true,
                "preferences": {}
            },
            {
                "id": 2,
                "name": "Bob",
                "email": "bob@example.com",
                "active": false,
                "preferences": {}
            }
        ]
    "#;
    
    let users = parse_json_array(users_json)?;
    println!("Parsed {} users", users.len());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_json_parsing() {
        let json = r#"{"id": 100, "name": "Test", "email": "test@example.com", "active": true, "preferences": {}}"#;
        let result: Result<User> = serde_json::from_str(json);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_invalid_email() {
        let json = r#"{"id": 100, "name": "Test", "email": "", "active": true, "preferences": {}}"#;
        let result: Result<User> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

pub struct JsonParser {
    input: Vec<char>,
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        match self.input[self.position] {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", self.input[self.position])),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.position + 3 < self.input.len() {
            let slice: String = self.input[self.position..self.position + 4].iter().collect();
            if slice == "null" {
                self.position += 4;
                return Ok(JsonValue::Null);
            }
        }
        Err("Expected 'null'".to_string())
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.position + 3 < self.input.len() {
            let slice: String = self.input[self.position..self.position + 4].iter().collect();
            if slice == "true" {
                self.position += 4;
                return Ok(JsonValue::Bool(true));
            }
        }
        if self.position + 4 < self.input.len() {
            let slice: String = self.input[self.position..self.position + 5].iter().collect();
            if slice == "false" {
                self.position += 5;
                return Ok(JsonValue::Bool(false));
            }
        }
        Err("Expected 'true' or 'false'".to_string())
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.position += 1; // Skip opening quote
        let mut result = String::new();
        
        while self.position < self.input.len() && self.input[self.position] != '"' {
            result.push(self.input[self.position]);
            self.position += 1;
        }
        
        if self.position < self.input.len() && self.input[self.position] == '"' {
            self.position += 1; // Skip closing quote
            Ok(JsonValue::String(result))
        } else {
            Err("Unterminated string".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        
        if self.input[self.position] == '-' {
            self.position += 1;
        }
        
        while self.position < self.input.len() && self.input[self.position].is_ascii_digit() {
            self.position += 1;
        }
        
        if self.position < self.input.len() && self.input[self.position] == '.' {
            self.position += 1;
            while self.position < self.input.len() && self.input[self.position].is_ascii_digit() {
                self.position += 1;
            }
        }
        
        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.position += 1; // Skip '['
        let mut array = Vec::new();
        
        self.skip_whitespace();
        
        if self.position < self.input.len() && self.input[self.position] == ']' {
            self.position += 1;
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if self.position >= self.input.len() {
                return Err("Unterminated array".to_string());
            }
            
            if self.input[self.position] == ']' {
                self.position += 1;
                break;
            } else if self.input[self.position] == ',' {
                self.position += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or ']', found: {}", self.input[self.position]));
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.position += 1; // Skip '{'
        let mut object = HashMap::new();
        
        self.skip_whitespace();
        
        if self.position < self.input.len() && self.input[self.position] == '}' {
            self.position += 1;
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            if self.input[self.position] != '"' {
                return Err("Expected string key".to_string());
            }
            
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };
            
            self.skip_whitespace();
            if self.position >= self.input.len() || self.input[self.position] != ':' {
                return Err("Expected ':'".to_string());
            }
            self.position += 1;
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            if self.position >= self.input.len() {
                return Err("Unterminated object".to_string());
            }
            
            if self.input[self.position] == '}' {
                self.position += 1;
                break;
            } else if self.input[self.position] == ',' {
                self.position += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or '}}', found: {}", self.input[self.position]));
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null");
        assert_eq!(parser.parse(), Ok(JsonValue::Null));
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(true)));
        
        let mut parser = JsonParser::new("false");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(false)));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));
        
        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new("\"hello world\"");
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"name": "test", "value": 42}"#);
        let mut expected_map = HashMap::new();
        expected_map.insert("name".to_string(), JsonValue::String("test".to_string()));
        expected_map.insert("value".to_string(), JsonValue::Number(42.0));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse(), Ok(expected));
    }
}