use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug)]
pub struct JsonParseError {
    message: String,
    position: usize,
}

impl fmt::Display for JsonParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON parse error at position {}: {}", self.position, self.message)
    }
}

impl Error for JsonParseError {}

pub struct JsonParser {
    input: Vec<char>,
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, JsonParseError> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err(JsonParseError {
                message: "Unexpected trailing characters".to_string(),
                position: self.position,
            });
        }
        
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, JsonParseError> {
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(JsonParseError {
                message: "Unexpected character".to_string(),
                position: self.position,
            }),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, JsonParseError> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err(JsonParseError {
                message: "Expected 'null'".to_string(),
                position: self.position,
            })
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, JsonParseError> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(JsonParseError {
                message: "Expected boolean value".to_string(),
                position: self.position,
            })
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, JsonParseError> {
        self.consume_char(); // Skip opening quote
        
        let mut result = String::new();
        while let Some(c) = self.peek_char() {
            if c == '"' {
                self.consume_char(); // Skip closing quote
                return Ok(JsonValue::String(result));
            }
            
            if c == '\\' {
                self.consume_char();
                if let Some(escaped) = self.peek_char() {
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err(JsonParseError {
                            message: format!("Invalid escape sequence: \\{}", escaped),
                            position: self.position - 1,
                        }),
                    }
                    self.consume_char();
                }
            } else {
                result.push(c);
                self.consume_char();
            }
        }
        
        Err(JsonParseError {
            message: "Unterminated string".to_string(),
            position: self.position,
        })
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonParseError> {
        let start = self.position;
        let mut has_dot = false;
        
        if self.peek_char() == Some('-') {
            self.consume_char();
        }
        
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.consume_char();
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.consume_char();
            } else {
                break;
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(JsonParseError {
                message: "Invalid number format".to_string(),
                position: start,
            }),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonParseError> {
        self.consume_char(); // Skip '['
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek_char() == Some(']') {
            self.consume_char();
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            
            match self.peek_char() {
                Some(',') => {
                    self.consume_char();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.consume_char();
                    break;
                }
                _ => {
                    return Err(JsonParseError {
                        message: "Expected ',' or ']' in array".to_string(),
                        position: self.position,
                    });
                }
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonParseError> {
        self.consume_char(); // Skip '{'
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.peek_char() == Some('}') {
            self.consume_char();
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            
            if self.peek_char() != Some('"') {
                return Err(JsonParseError {
                    message: "Expected string key in object".to_string(),
                    position: self.position,
                });
            }
            
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            
            if self.peek_char() != Some(':') {
                return Err(JsonParseError {
                    message: "Expected ':' after object key".to_string(),
                    position: self.position,
                });
            }
            
            self.consume_char(); // Skip ':'
            self.skip_whitespace();
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            
            match self.peek_char() {
                Some(',') => {
                    self.consume_char();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.consume_char();
                    break;
                }
                _ => {
                    return Err(JsonParseError {
                        message: "Expected ',' or '}' in object".to_string(),
                        position: self.position,
                    });
                }
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn consume_char(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            Some(c)
        } else {
            None
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.position + chars.len() <= self.input.len() {
            for (i, &expected) in chars.iter().enumerate() {
                if self.input[self.position + i] != expected {
                    return false;
                }
            }
            self.position += chars.len();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.consume_char();
            } else {
                break;
            }
        }
    }
}

pub fn validate_json(input: &str) -> Result<JsonValue, JsonParseError> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let json = r#"{"name": "test", "value": 42.5, "active": true, "tags": ["rust", "json"]}"#;
        let result = validate_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{"name": test}"#;
        let result = validate_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_object() {
        let json = r#"{}"#;
        let result = validate_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_escape_sequences() {
        let json = r#"{"text": "line1\nline2\t\"quoted\""}"#;
        let result = validate_json(json);
        assert!(result.is_ok());
    }
}