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

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected boolean value".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.expect_char('"')?;
        let mut result = String::new();
        
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.next_char().ok_or("Unterminated escape sequence")?;
                    result.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'b' => '\x08',
                        'f' => '\x0c',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        _ => return Err("Invalid escape sequence".to_string()),
                    });
                }
                _ => result.push(c),
            }
        }
        
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;
        let mut has_exp = false;
        
        if self.peek_char() == Some('-') {
            self.next_char();
        }
        
        while let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => { self.next_char(); }
                '.' if !has_dot && !has_exp => {
                    has_dot = true;
                    self.next_char();
                    if !self.peek_char().map_or(false, |c| c.is_digit(10)) {
                        return Err("Invalid number format".to_string());
                    }
                }
                'e' | 'E' if !has_exp => {
                    has_exp = true;
                    self.next_char();
                    if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                        self.next_char();
                    }
                }
                _ => break,
            }
        }
        
        let num_str = &self.input[start..self.pos];
        num_str.parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| "Invalid number format".to_string())
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect_char('[')?;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek_char() != Some(']') {
            loop {
                let value = self.parse_value()?;
                array.push(value);
                
                self.skip_whitespace();
                if self.peek_char() == Some(']') {
                    break;
                }
                
                self.expect_char(',')?;
                self.skip_whitespace();
            }
        }
        
        self.expect_char(']')?;
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect_char('{')?;
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.peek_char() != Some('}') {
            loop {
                let key = match self.parse_value()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Object keys must be strings".to_string()),
                };
                
                self.skip_whitespace();
                self.expect_char(':')?;
                self.skip_whitespace();
                
                let value = self.parse_value()?;
                object.insert(key, value);
                
                self.skip_whitespace();
                if self.peek_char() == Some('}') {
                    break;
                }
                
                self.expect_char(',')?;
                self.skip_whitespace();
            }
        }
        
        self.expect_char('}')?;
        Ok(JsonValue::Object(object))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.next_char() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(format!("Expected '{}', found '{}'", expected, c)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn consume(&mut self, target: &str) -> bool {
        if self.input[self.pos..].starts_with(target) {
            self.pos += target.len();
            true
        } else {
            false
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