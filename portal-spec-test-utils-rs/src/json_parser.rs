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
pub struct ParseError {
    message: String,
    position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON parse error at position {}: {}", self.position, self.message)
    }
}

impl Error for ParseError {}

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

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err(self.error("Unexpected trailing characters"));
        }
        
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(self.error("Expected JSON value")),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("null")?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.starts_with("true") {
            self.advance(4);
            Ok(JsonValue::Bool(true))
        } else if self.starts_with("false") {
            self.advance(5);
            Ok(JsonValue::Bool(false))
        } else {
            Err(self.error("Expected boolean value"))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_decimal = false;
        
        if self.peek_char() == Some('-') {
            self.advance(1);
        }
        
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.advance(1);
            } else if c == '.' && !has_decimal {
                has_decimal = true;
                self.advance(1);
            } else {
                break;
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(self.error("Invalid number format")),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("\"")?;
        let mut result = String::new();
        
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.next_char().ok_or_else(|| self.error("Unterminated escape sequence"))?;
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err(self.error("Invalid escape sequence")),
                    }
                }
                _ => result.push(c),
            }
        }
        
        Ok(JsonValue::String(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("[")?;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek_char() == Some(']') {
            self.advance(1);
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.advance(1);
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.advance(1);
                    break;
                }
                _ => return Err(self.error("Expected ',' or ']' in array")),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("{")?;
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.peek_char() == Some('}') {
            self.advance(1);
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            self.expect(":")?;
            self.skip_whitespace();
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.advance(1);
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.advance(1);
                    break;
                }
                _ => return Err(self.error("Expected ',' or '}' in object")),
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance(1);
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn advance(&mut self, n: usize) {
        self.position += n;
    }

    fn starts_with(&self, s: &str) -> bool {
        let end = self.position + s.len();
        if end > self.input.len() {
            return false;
        }
        self.input[self.position..end].iter().collect::<String>() == s
    }

    fn expect(&mut self, expected: &str) -> Result<(), ParseError> {
        if self.starts_with(expected) {
            self.advance(expected.len());
            Ok(())
        } else {
            Err(self.error(&format!("Expected '{}'", expected)))
        }
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            position: self.position,
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
        let mut parser = JsonParser::new("\"hello world\"");
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
        
        let mut parser = JsonParser::new("\"escaped \\\"quote\\\"\"");
        assert_eq!(parser.parse(), Ok(JsonValue::String("escaped \"quote\"".to_string())));
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
        let mut parser = JsonParser::new(r#"{"key": "value", "number": 42}"#);
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        expected_map.insert("number".to_string(), JsonValue::Number(42.0));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse(), Ok(expected));
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

#[derive(Debug)]
pub struct ParseError {
    message: String,
    position: usize,
}

impl ParseError {
    fn new(msg: &str, pos: usize) -> Self {
        ParseError {
            message: msg.to_string(),
            position: pos,
        }
    }
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

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some(ch) if ch == expected => {
                self.position += 1;
                Ok(())
            }
            Some(ch) => Err(ParseError::new(
                &format!("Expected '{}', found '{}'", expected, ch),
                self.position,
            )),
            None => Err(ParseError::new(
                &format!("Expected '{}', found EOF", expected),
                self.position,
            )),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('n') => self.parse_null(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err(ParseError::new("Unexpected token", self.position)),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.consume('{')?;
        self.skip_whitespace();

        let mut map = HashMap::new();

        if let Some('}') = self.peek() {
            self.consume('}')?;
            return Ok(JsonValue::Object(map));
        }

        loop {
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };

            self.consume(':')?;
            let value = self.parse()?;
            map.insert(key, value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume(',')?;
                    continue;
                }
                Some('}') => {
                    self.consume('}')?;
                    break;
                }
                _ => return Err(ParseError::new("Expected ',' or '}'", self.position)),
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.consume('[')?;
        self.skip_whitespace();

        let mut arr = Vec::new();

        if let Some(']') = self.peek() {
            self.consume(']')?;
            return Ok(JsonValue::Array(arr));
        }

        loop {
            let value = self.parse()?;
            arr.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume(',')?;
                    continue;
                }
                Some(']') => {
                    self.consume(']')?;
                    break;
                }
                _ => return Err(ParseError::new("Expected ',' or ']'", self.position)),
            }
        }

        Ok(JsonValue::Array(arr))
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.consume('"')?;
        let start = self.position;
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.position += 1;
                return Ok(JsonValue::String(result));
            }
            if ch == '\\' {
                result.push_str(&self.input[start..self.position].iter().collect::<String>());
                self.position += 1;
                match self.peek() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\x08'),
                    Some('f') => result.push('\x0c'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(_) => return Err(ParseError::new("Invalid escape sequence", self.position)),
                    None => return Err(ParseError::new("Unexpected EOF", self.position)),
                }
                self.position += 1;
                continue;
            }
            self.position += 1;
        }

        Err(ParseError::new("Unterminated string", start))
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        match self.peek() {
            Some('t') => {
                if self.position + 3 < self.input.len()
                    && self.input[self.position..self.position + 4] == ['t', 'r', 'u', 'e']
                {
                    self.position += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err(ParseError::new("Expected 'true'", start))
                }
            }
            Some('f') => {
                if self.position + 4 < self.input.len()
                    && self.input[self.position..self.position + 5] == ['f', 'a', 'l', 's', 'e']
                {
                    self.position += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err(ParseError::new("Expected 'false'", start))
                }
            }
            _ => Err(ParseError::new("Expected boolean", start)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        if self.position + 3 < self.input.len()
            && self.input[self.position..self.position + 4] == ['n', 'u', 'l', 'l']
        {
            self.position += 4;
            Ok(JsonValue::Null)
        } else {
            Err(ParseError::new("Expected 'null'", start))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_dot = false;
        let mut has_exp = false;

        if self.peek() == Some('-') {
            self.position += 1;
        }

        while let Some(ch) = self.peek() {
            match ch {
                '0'..='9' => {
                    self.position += 1;
                }
                '.' => {
                    if has_dot || has_exp {
                        break;
                    }
                    has_dot = true;
                    self.position += 1;
                }
                'e' | 'E' => {
                    if has_exp {
                        break;
                    }
                    has_exp = true;
                    self.position += 1;
                    if self.peek() == Some('-') || self.peek() == Some('+') {
                        self.position += 1;
                    }
                }
                _ => break,
            }
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError::new("Invalid number", start)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let mut parser = JsonParser::new(r#"{"name": "test", "value": 42}"#);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new(r#"[1, 2, 3, true, false, null]"#);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""Hello, World!""#);
        let result = parser.parse();
        assert!(matches!(result, Ok(JsonValue::String(_))));
    }
}