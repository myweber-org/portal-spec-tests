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
        let result = match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err("Invalid JSON token".to_string()),
        }?;
        self.skip_whitespace();
        Ok(result)
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        self.consume('n')?;
        self.consume('u')?;
        self.consume('l')?;
        self.consume('l')?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        match self.peek() {
            Some('t') => {
                self.consume('t')?;
                self.consume('r')?;
                self.consume('u')?;
                self.consume('e')?;
                Ok(JsonValue::Bool(true))
            }
            Some('f') => {
                self.consume('f')?;
                self.consume('a')?;
                self.consume('l')?;
                self.consume('s')?;
                self.consume('e')?;
                Ok(JsonValue::Bool(false))
            }
            _ => Err("Expected boolean value".to_string()),
        }
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
            Err(_) => Err("Invalid number format".to_string()),
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

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut items = Vec::new();
        if self.peek() != Some(']') {
            loop {
                let value = self.parse()?;
                items.push(value);
                self.skip_whitespace();
                if self.peek() == Some(',') {
                    self.consume(',')?;
                    self.skip_whitespace();
                } else {
                    break;
                }
            }
        }
        self.consume(']')?;
        Ok(JsonValue::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek() != Some('}') {
            loop {
                let key = match self.parse()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Object key must be a string".to_string()),
                };
                self.skip_whitespace();
                self.consume(':')?;
                self.skip_whitespace();
                let value = self.parse()?;
                map.insert(key, value);
                self.skip_whitespace();
                if self.peek() == Some(',') {
                    self.consume(',')?;
                    self.skip_whitespace();
                } else {
                    break;
                }
            }
        }
        self.consume('}')?;
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
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));
        
        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new("\"hello\"");
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello".to_string())));
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
        let expected = JsonValue::Object(map);
        assert_eq!(parser.parse(), Ok(expected));
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug)]
struct ParseError {
    message: String,
    position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl Error for ParseError {}

struct JsonParser {
    input: Vec<char>,
    position: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err(ParseError {
                message: "Unexpected trailing characters".to_string(),
                position: self.position,
            });
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(ParseError {
                message: "Unexpected character".to_string(),
                position: self.position,
            }),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("null")?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.starts_with("true") {
            self.position += 4;
            Ok(JsonValue::Bool(true))
        } else if self.starts_with("false") {
            self.position += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(ParseError {
                message: "Expected boolean".to_string(),
                position: self.position,
            })
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
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
        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError {
                message: "Invalid number format".to_string(),
                position: start,
            }),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("\"")?;
        let mut result = String::new();
        while let Some(c) = self.peek_char() {
            if c == '"' {
                break;
            } else if c == '\\' {
                self.advance();
                match self.peek_char() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\u{0008}'),
                    Some('f') => result.push('\u{000C}'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(_) => {
                        return Err(ParseError {
                            message: "Invalid escape sequence".to_string(),
                            position: self.position,
                        })
                    }
                    None => {
                        return Err(ParseError {
                            message: "Unterminated string".to_string(),
                            position: self.position,
                        })
                    }
                }
                self.advance();
            } else {
                result.push(c);
                self.advance();
            }
        }
        self.expect("\"")?;
        Ok(JsonValue::String(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("[")?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if self.peek_char() == Some(']') {
            self.advance();
            return Ok(JsonValue::Array(array));
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                self.advance();
                break;
            }
            self.expect(",")?;
            self.skip_whitespace();
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("{")?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek_char() == Some('}') {
            self.advance();
            return Ok(JsonValue::Object(map));
        }
        loop {
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            self.skip_whitespace();
            self.expect(":")?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if self.peek_char() == Some('}') {
                self.advance();
                break;
            }
            self.expect(",")?;
            self.skip_whitespace();
        }
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
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        let end = self.position + s.len();
        if end > self.input.len() {
            return false;
        }
        self.input[self.position..end]
            .iter()
            .collect::<String>()
            == s
    }

    fn expect(&mut self, expected: &str) -> Result<(), ParseError> {
        if self.starts_with(expected) {
            self.position += expected.len();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected '{}'", expected),
                position: self.position,
            })
        }
    }
}

fn parse_json(json_str: &str) -> Result<JsonValue, ParseError> {
    let mut parser = JsonParser::new(json_str);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        assert_eq!(parse_json("null").unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_json("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse_json("false").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_json("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse_json("-3.14").unwrap(), JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_json("\"hello\"").unwrap(),
            JsonValue::String("hello".to_string())
        );
        assert_eq!(
            parse_json("\"escaped\\nstring\"").unwrap(),
            JsonValue::String("escaped\nstring".to_string())
        );
    }

    #[test]
    fn test_parse_array() {
        let result = parse_json("[1, true, \"test\"]").unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::Bool(true));
            assert_eq!(arr[2], JsonValue::String("test".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_object() {
        let result = parse_json("{\"key\": \"value\", \"num\": 123}").unwrap();
        if let JsonValue::Object(map) = result {
            assert_eq!(map.len(), 2);
            assert_eq!(
                map.get("key"),
                Some(&JsonValue::String("value".to_string()))
            );
            assert_eq!(map.get("num"), Some(&JsonValue::Number(123.0)));
        } else {
            panic!("Expected object");
        }
    }
}