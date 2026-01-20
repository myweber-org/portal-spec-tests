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

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        let ch = self.input[self.pos];
        match ch {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", ch)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip opening quote
        let mut result = String::new();

        while self.pos < self.input.len() && self.input[self.pos] != '"' {
            result.push(self.input[self.pos]);
            self.pos += 1;
        }

        if self.pos < self.input.len() && self.input[self.pos] == '"' {
            self.pos += 1; // Skip closing quote
            Ok(JsonValue::String(result))
        } else {
            Err("Unterminated string".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.input[self.pos] == '-' {
            self.pos += 1;
        }

        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        if self.pos < self.input.len() && self.input[self.pos] == '.' {
            self.pos += 1;
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '['
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.pos < self.input.len() && self.input[self.pos] == ']' {
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

            if self.input[self.pos] == ']' {
                self.pos += 1;
                break;
            } else if self.input[self.pos] == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err("Expected ',' or ']'".to_string());
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '{'
        self.skip_whitespace();
        let mut object = HashMap::new();

        if self.pos < self.input.len() && self.input[self.pos] == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input[self.pos] != '"' {
                return Err("Expected string key".to_string());
            }

            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input[self.pos] != ':' {
                return Err("Expected ':'".to_string());
            }
            self.pos += 1;

            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                return Err("Unterminated object".to_string());
            }

            if self.input[self.pos] == '}' {
                self.pos += 1;
                break;
            } else if self.input[self.pos] == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err("Expected ',' or '}'".to_string());
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.pos + chars.len() <= self.input.len() {
            for (i, &ch) in chars.iter().enumerate() {
                if self.input[self.pos + i] != ch {
                    return false;
                }
            }
            self.pos += chars.len();
            true
        } else {
            false
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
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));
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
pub enum ParseError {
    UnexpectedEnd,
    InvalidToken,
    ExpectedColon,
    ExpectedComma,
    TrailingComma,
    InvalidNumber,
    InvalidEscape,
    UnterminatedString,
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

    fn parse_string(&mut self) -> Result<String, ParseError> {
        let mut result = String::new();
        self.consume(); // consume opening quote

        while let Some(ch) = self.consume() {
            match ch {
                '"' => return Ok(result),
                '\\' => {
                    let escaped = self.consume().ok_or(ParseError::UnexpectedEnd)?;
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err(ParseError::InvalidEscape),
                    }
                }
                _ => result.push(ch),
            }
        }

        Err(ParseError::UnterminatedString)
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.position;
        let mut has_dot = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.consume();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.consume();
            } else {
                break;
            }
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        num_str.parse().map_err(|_| ParseError::InvalidNumber)
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, ParseError> {
        let mut array = Vec::new();
        self.consume(); // consume opening bracket

        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.consume();
            return Ok(array);
        }

        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume();
                    self.skip_whitespace();
                    if self.peek() == Some(']') {
                        return Err(ParseError::TrailingComma);
                    }
                }
                Some(']') => {
                    self.consume();
                    break;
                }
                _ => return Err(ParseError::ExpectedComma),
            }
        }

        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, ParseError> {
        let mut map = HashMap::new();
        self.consume(); // consume opening brace

        self.skip_whitespace();
        if self.peek() == Some('}') {
            self.consume();
            return Ok(map);
        }

        loop {
            self.skip_whitespace();
            if self.peek() != Some('"') {
                return Err(ParseError::InvalidToken);
            }

            let key = self.parse_string()?;
            self.skip_whitespace();

            if self.peek() != Some(':') {
                return Err(ParseError::ExpectedColon);
            }
            self.consume();

            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume();
                    self.skip_whitespace();
                    if self.peek() == Some('}') {
                        return Err(ParseError::TrailingComma);
                    }
                }
                Some('}') => {
                    self.consume();
                    break;
                }
                _ => return Err(ParseError::ExpectedComma),
            }
        }

        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some('n') => {
                if self.consume() == Some('n')
                    && self.consume() == Some('u')
                    && self.consume() == Some('l')
                    && self.consume() == Some('l')
                {
                    Ok(JsonValue::Null)
                } else {
                    Err(ParseError::InvalidToken)
                }
            }
            Some('t') => {
                if self.consume() == Some('t')
                    && self.consume() == Some('r')
                    && self.consume() == Some('u')
                    && self.consume() == Some('e')
                {
                    Ok(JsonValue::Bool(true))
                } else {
                    Err(ParseError::InvalidToken)
                }
            }
            Some('f') => {
                if self.consume() == Some('f')
                    && self.consume() == Some('a')
                    && self.consume() == Some('l')
                    && self.consume() == Some('s')
                    && self.consume() == Some('e')
                {
                    Ok(JsonValue::Bool(false))
                } else {
                    Err(ParseError::InvalidToken)
                }
            }
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('[') => self.parse_array().map(JsonValue::Array),
            Some('{') => self.parse_object().map(JsonValue::Object),
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                self.parse_number().map(JsonValue::Number)
            }
            _ => Err(ParseError::InvalidToken),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.peek().is_some() {
            Err(ParseError::InvalidToken)
        } else {
            Ok(result)
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
        let mut parser = JsonParser::new(r#"[1, 2, 3, "four"]"#);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let mut parser = JsonParser::new(r#"{"unclosed": string}"#);
        let result = parser.parse();
        assert!(result.is_err());
    }
}use std::collections::HashMap;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

pub struct JsonParser<'a> {
    chars: Chars<'a>,
    current: Option<char>,
}

impl<'a> JsonParser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        JsonParser { chars, current }
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.current {
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
        let expected = "null";
        for ch in expected.chars() {
            match self.current {
                Some(c) if c == ch => self.advance(),
                _ => return Err("Expected 'null'".to_string()),
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.current == Some('t') {
            let expected = "true";
            for ch in expected.chars() {
                match self.current {
                    Some(c) if c == ch => self.advance(),
                    _ => return Err("Expected 'true'".to_string()),
                }
            }
            Ok(JsonValue::Bool(true))
        } else {
            let expected = "false";
            for ch in expected.chars() {
                match self.current {
                    Some(c) if c == ch => self.advance(),
                    _ => return Err("Expected 'false'".to_string()),
                }
            }
            Ok(JsonValue::Bool(false))
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.advance(); // Skip opening quote
        let mut result = String::new();
        while let Some(c) = self.current {
            if c == '"' {
                self.advance();
                return Ok(JsonValue::String(result));
            }
            result.push(c);
            self.advance();
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut num_str = String::new();
        while let Some(c) = self.current {
            if c.is_digit(10) || c == '-' || c == '.' || c == 'e' || c == 'E' {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match num_str.parse::<f64>() {
            Ok(n) => Ok(JsonValue::Number(n)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.advance(); // Skip '['
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.current == Some(']') {
            self.advance();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            match self.current {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.advance();
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.advance(); // Skip '{'
        self.skip_whitespace();
        let mut map = HashMap::new();

        if self.current == Some('}') {
            self.advance();
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };

            self.skip_whitespace();
            match self.current {
                Some(':') => self.advance(),
                _ => return Err("Expected ':' after object key".to_string()),
            }

            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();

            match self.current {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.advance();
                    break;
                }
                _ => return Err("Expected ',' or '}' in object".to_string()),
            }
        }
        Ok(JsonValue::Object(map))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.current.is_some() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}