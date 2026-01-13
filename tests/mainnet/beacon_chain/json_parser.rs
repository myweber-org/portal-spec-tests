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

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn consume(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.position += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err(format!("Unexpected character at position {}", self.position)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for expected_char in expected.chars() {
            match self.consume() {
                Some(ch) if ch == expected_char => continue,
                _ => return Err(format!("Expected '{}'", expected)),
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        let first = self.consume().unwrap();
        if first == 't' {
            let expected = "rue";
            for expected_char in expected.chars() {
                match self.consume() {
                    Some(ch) if ch == expected_char => continue,
                    _ => return Err("Expected 'true'".to_string()),
                }
            }
            Ok(JsonValue::Bool(true))
        } else {
            let expected = "alse";
            for expected_char in expected.chars() {
                match self.consume() {
                    Some(ch) if ch == expected_char => continue,
                    _ => return Err("Expected 'false'".to_string()),
                }
            }
            Ok(JsonValue::Bool(false))
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume(); // Consume opening quote
        let mut result = String::new();
        
        while let Some(ch) = self.consume() {
            if ch == '"' {
                return Ok(JsonValue::String(result));
            }
            result.push(ch);
        }
        
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut num_str = String::new();
        
        if let Some('-') = self.peek() {
            num_str.push(self.consume().unwrap());
        }
        
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-' {
                num_str.push(self.consume().unwrap());
            } else {
                break;
            }
        }
        
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume(); // Consume '['
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if let Some(']') = self.peek() {
            self.consume();
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.consume();
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume(); // Consume '{'
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if let Some('}') = self.peek() {
            self.consume();
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            
            if let Some('"') = self.peek() {
                let key = match self.parse_string()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Expected string key".to_string()),
                };
                
                self.skip_whitespace();
                match self.consume() {
                    Some(':') => (),
                    _ => return Err("Expected ':' after object key".to_string()),
                }
                
                self.skip_whitespace();
                let value = self.parse_value()?;
                object.insert(key, value);
                
                self.skip_whitespace();
                match self.peek() {
                    Some(',') => {
                        self.consume();
                        self.skip_whitespace();
                    }
                    Some('}') => {
                        self.consume();
                        break;
                    }
                    _ => return Err("Expected ',' or '}' in object".to_string()),
                }
            } else {
                return Err("Expected string key in object".to_string());
            }
        }
        
        Ok(JsonValue::Object(object))
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
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
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
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse(), Ok(expected));
    }
}