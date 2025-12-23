
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub settings: HashMap<String, Value>,
    pub enabled: bool,
}

pub fn parse_json_file(file_path: &str) -> Result<Config> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| serde_json::Error::io(e))?;
    
    let config: Config = serde_json::from_str(&content)?;
    
    if config.name.is_empty() {
        return Err(serde_json::Error::custom("Name field cannot be empty"));
    }
    
    if config.version.is_empty() {
        return Err(serde_json::Error::custom("Version field cannot be empty"));
    }
    
    Ok(config)
}

pub fn validate_json_structure(json_str: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(json_str)?;
    
    if !value.is_object() {
        return Err(serde_json::Error::custom("JSON must be an object"));
    }
    
    Ok(value)
}

pub fn merge_json_objects(a: &Value, b: &Value) -> Result<Value> {
    if !a.is_object() || !b.is_object() {
        return Err(serde_json::Error::custom("Both values must be JSON objects"));
    }
    
    let mut merged = a.as_object().unwrap().clone();
    let b_obj = b.as_object().unwrap();
    
    for (key, value) in b_obj {
        merged.insert(key.clone(), value.clone());
    }
    
    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_json_structure() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(validate_json_structure(valid_json).is_ok());
        
        let invalid_json = r#"["array", "not", "object"]"#;
        assert!(validate_json_structure(invalid_json).is_err());
    }
    
    #[test]
    fn test_merge_json_objects() {
        let json_a = serde_json::json!({"a": 1, "b": 2});
        let json_b = serde_json::json!({"b": 3, "c": 4});
        
        let result = merge_json_objects(&json_a, &json_b).unwrap();
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 3);
        assert_eq!(result["c"], 4);
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

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.pos += 1;
                Ok(())
            } else {
                Err(format!("Expected '{}', found '{}'", expected, ch))
            }
        } else {
            Err(format!("Expected '{}', found EOF", expected))
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.pos += 1;
                return Ok(result);
            } else if ch == '\\' {
                self.pos += 1;
                if let Some(escaped) = self.peek() {
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
                    return Err("Unterminated escape sequence".to_string());
                }
            } else {
                result.push(ch);
                self.pos += 1;
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == 'e' || ch == 'E' || ch == '+' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        num_str
            .parse::<f64>()
            .map_err(|_| format!("Invalid number: {}", num_str))
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if let Some(ch) = self.peek() {
            if ch == ']' {
                self.pos += 1;
                return Ok(array);
            }
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == ']' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    self.skip_whitespace();
                } else {
                    return Err(format!("Expected ',' or ']', found '{}'", ch));
                }
            } else {
                return Err("Unterminated array".to_string());
            }
        }
        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if let Some(ch) = self.peek() {
            if ch == '}' {
                self.pos += 1;
                return Ok(map);
            }
        }
        loop {
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == '}' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    self.skip_whitespace();
                } else {
                    return Err(format!("Expected ',' or '}}', found '{}'", ch));
                }
            } else {
                return Err("Unterminated object".to_string());
            }
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            match ch {
                'n' => {
                    if self.input.len() >= self.pos + 4
                        && self.input[self.pos..self.pos + 4]
                            .iter()
                            .collect::<String>()
                            == "null"
                    {
                        self.pos += 4;
                        Ok(JsonValue::Null)
                    } else {
                        Err("Expected 'null'".to_string())
                    }
                }
                't' => {
                    if self.input.len() >= self.pos + 4
                        && self.input[self.pos..self.pos + 4]
                            .iter()
                            .collect::<String>()
                            == "true"
                    {
                        self.pos += 4;
                        Ok(JsonValue::Bool(true))
                    } else {
                        Err("Expected 'true'".to_string())
                    }
                }
                'f' => {
                    if self.input.len() >= self.pos + 5
                        && self.input[self.pos..self.pos + 5]
                            .iter()
                            .collect::<String>()
                            == "false"
                    {
                        self.pos += 5;
                        Ok(JsonValue::Bool(false))
                    } else {
                        Err("Expected 'false'".to_string())
                    }
                }
                '"' => self.parse_string().map(JsonValue::String),
                '[' => self.parse_array().map(JsonValue::Array),
                '{' => self.parse_object().map(JsonValue::Object),
                '-' | '0'..='9' => self.parse_number().map(JsonValue::Number),
                _ => Err(format!("Unexpected character: {}", ch)),
            }
        } else {
            Err("Unexpected EOF".to_string())
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            Err("Trailing characters after JSON value".to_string())
        } else {
            Ok(result)
        }
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
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
        );
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