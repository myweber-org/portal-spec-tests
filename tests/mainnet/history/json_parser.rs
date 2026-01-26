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
            Err("Expected null".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected boolean".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek_char() == Some('-') {
            self.advance();
        }
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.advance();
            } else {
                break;
            }
        }
        if self.peek_char() == Some('.') {
            self.advance();
            while let Some(c) = self.peek_char() {
                if c.is_digit(10) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.expect('"')?;
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.next_char().ok_or("Unterminated escape sequence")?;
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err("Invalid escape sequence".to_string()),
                    }
                }
                _ => result.push(c),
            }
        }
        Ok(JsonValue::String(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect('[')?;
        self.skip_whitespace();
        let mut items = Vec::new();
        if self.peek_char() != Some(']') {
            loop {
                let value = self.parse_value()?;
                items.push(value);
                self.skip_whitespace();
                if self.peek_char() == Some(']') {
                    break;
                }
                self.expect(',')?;
                self.skip_whitespace();
            }
        }
        self.expect(']')?;
        Ok(JsonValue::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek_char() != Some('}') {
            loop {
                let key = match self.parse_string()? {
                    JsonValue::String(s) => s,
                    _ => unreachable!(),
                };
                self.skip_whitespace();
                self.expect(':')?;
                let value = self.parse_value()?;
                map.insert(key, value);
                self.skip_whitespace();
                if self.peek_char() == Some('}') {
                    break;
                }
                self.expect(',')?;
                self.skip_whitespace();
            }
        }
        self.expect('}')?;
        Ok(JsonValue::Object(map))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance();
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

    fn advance(&mut self) {
        if self.pos < self.input.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        if let Some(c) = self.next_char() {
            if c == expected {
                Ok(())
            } else {
                Err(format!("Expected '{}', found '{}'", expected, c))
            }
        } else {
            Err("Unexpected end of input".to_string())
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
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));
        
        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
        
        let mut parser = JsonParser::new(r#""escape\nsequence""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("escape\nsequence".to_string())));
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
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
    }
}