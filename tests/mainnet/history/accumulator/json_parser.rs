use serde_json::{Value, Error};
use std::collections::HashMap;
use std::fs;

pub struct JsonParser {
    data: HashMap<String, Value>,
}

impl JsonParser {
    pub fn new() -> Self {
        JsonParser {
            data: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Error> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::io(e))?;
        
        let parsed: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(map) = parsed {
            for (key, value) in map {
                self.data.insert(key, value);
            }
            Ok(())
        } else {
            Err(Error::custom("Root must be a JSON object"))
        }
    }

    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn validate_schema(&self, schema: &HashMap<String, &str>) -> Vec<String> {
        let mut errors = Vec::new();
        
        for (key, expected_type) in schema {
            match self.data.get(key) {
                Some(value) => {
                    let actual_type = match value {
                        Value::Null => "null",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Array(_) => "array",
                        Value::Object(_) => "object",
                    };
                    
                    if actual_type != *expected_type {
                        errors.push(format!(
                            "Field '{}': expected type '{}', found '{}'",
                            key, expected_type, actual_type
                        ));
                    }
                }
                None => {
                    errors.push(format!("Missing required field: '{}'", key));
                }
            }
        }
        
        errors
    }

    pub fn extract_strings(&self) -> Vec<String> {
        let mut strings = Vec::new();
        
        for value in self.data.values() {
            Self::collect_strings(value, &mut strings);
        }
        
        strings
    }

    fn collect_strings(value: &Value, strings: &mut Vec<String>) {
        match value {
            Value::String(s) => strings.push(s.clone()),
            Value::Array(arr) => {
                for item in arr {
                    Self::collect_strings(item, strings);
                }
            }
            Value::Object(obj) => {
                for val in obj.values() {
                    Self::collect_strings(val, strings);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parser_operations() {
        let mut parser = JsonParser::new();
        
        let test_json = json!({
            "name": "test",
            "count": 42,
            "tags": ["rust", "json"],
            "metadata": {
                "version": "1.0"
            }
        });
        
        let temp_file = "test_temp.json";
        fs::write(temp_file, test_json.to_string()).unwrap();
        
        let result = parser.load_from_file(temp_file);
        assert!(result.is_ok());
        
        let schema = HashMap::from([
            ("name".to_string(), "string"),
            ("count".to_string(), "number"),
        ]);
        
        let errors = parser.validate_schema(&schema);
        assert!(errors.is_empty());
        
        let strings = parser.extract_strings();
        assert!(strings.contains(&"test".to_string()));
        assert!(strings.contains(&"rust".to_string()));
        
        fs::remove_file(temp_file).unwrap();
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
    pos: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        match self.peek() {
            Some(ch) if ch == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(ch) => Err(format!("Expected '{}', found '{}'", expected, ch)),
            None => Err(format!("Expected '{}', found EOF", expected)),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err("Invalid JSON".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for ch in expected.chars() {
            self.consume(ch)?;
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
            for ch in "true".chars() {
                self.consume(ch)?;
            }
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
            for ch in "false".chars() {
                self.consume(ch)?;
            }
            Ok(JsonValue::Bool(false))
        } else {
            Err("Invalid boolean".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                break;
            }
            result.push(ch);
            self.pos += 1;
        }
        self.consume('"')?;
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.peek() == Some('.') {
            self.pos += 1;
            while let Some(ch) = self.peek() {
                if ch.is_digit(10) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if self.peek() == Some(']') {
            self.consume(']')?;
            return Ok(JsonValue::Array(array));
        }
        loop {
            let value = self.parse()?;
            array.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume(',')?;
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.consume(']')?;
                    break;
                }
                _ => return Err("Expected ',' or ']'".to_string()),
            }
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek() == Some('}') {
            self.consume('}')?;
            return Ok(JsonValue::Object(map));
        }
        loop {
            let key = match self.parse()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be string".to_string()),
            };
            self.skip_whitespace();
            self.consume(':')?;
            self.skip_whitespace();
            let value = self.parse()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume(',')?;
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.consume('}')?;
                    break;
                }
                _ => return Err("Expected ',' or '}'".to_string()),
            }
        }
        Ok(JsonValue::Object(map))
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
        let mut parser = JsonParser::new("\"hello\"");
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));
        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
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
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
    }
}