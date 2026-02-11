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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedCharacter(char, usize),
    UnexpectedEndOfInput,
    InvalidNumber,
    InvalidEscapeSequence,
    KeyMustBeString,
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

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ));
        }
        
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            Some(c) => Err(ParseError::UnexpectedCharacter(c, self.position)),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ))
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_decimal = false;
        
        if self.consume_char('-') {
            // Negative number, continue parsing
        }
        
        if !self.consume_digits() {
            return Err(ParseError::InvalidNumber);
        }
        
        if self.consume_char('.') {
            has_decimal = true;
            if !self.consume_digits() {
                return Err(ParseError::InvalidNumber);
            }
        }
        
        if self.consume_char('e') || self.consume_char('E') {
            self.consume_char('-');
            self.consume_char('+');
            if !self.consume_digits() {
                return Err(ParseError::InvalidNumber);
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError::InvalidNumber),
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect_char('"')?;
        let mut result = String::new();
        
        while let Some(c) = self.next_char() {
            match c {
                '"' => return Ok(result),
                '\\' => {
                    let escaped = self.parse_escape_sequence()?;
                    result.push(escaped);
                }
                c if c.is_control() => {
                    return Err(ParseError::UnexpectedCharacter(c, self.position - 1));
                }
                c => result.push(c),
            }
        }
        
        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_escape_sequence(&mut self) -> Result<char, ParseError> {
        match self.next_char() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\x08'),
            Some('f') => Ok('\x0c'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('u') => self.parse_unicode_escape(),
            Some(c) => Err(ParseError::InvalidEscapeSequence),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, ParseError> {
        let hex_str: String = (0..4)
            .filter_map(|_| self.next_char())
            .filter(|c| c.is_digit(16))
            .collect();
        
        if hex_str.len() != 4 {
            return Err(ParseError::InvalidEscapeSequence);
        }
        
        let code_point = u32::from_str_radix(&hex_str, 16)
            .map_err(|_| ParseError::InvalidEscapeSequence)?;
        
        char::from_u32(code_point).ok_or(ParseError::InvalidEscapeSequence)
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect_char('[')?;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.consume_char(']') {
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if self.consume_char(']') {
                break;
            }
            
            self.expect_char(',')?;
            self.skip_whitespace();
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect_char('{')?;
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.consume_char('}') {
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            
            self.skip_whitespace();
            self.expect_char(':')?;
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            if self.consume_char('}') {
                break;
            }
            
            self.expect_char(',')?;
            self.skip_whitespace();
        }
        
        Ok(JsonValue::Object(object))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.position += 1;
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

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        let expected_chars: Vec<char> = expected.chars().collect();
        if self.position + expected_chars.len() <= self.input.len() {
            let slice = &self.input[self.position..self.position + expected_chars.len()];
            if slice == expected_chars {
                self.position += expected_chars.len();
                return true;
            }
        }
        false
    }

    fn consume_digits(&mut self) -> bool {
        let start = self.position;
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.position += 1;
            } else {
                break;
            }
        }
        self.position > start
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        match self.next_char() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(ParseError::UnexpectedCharacter(c, self.position - 1)),
            None => Err(ParseError::UnexpectedEndOfInput),
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
        
        let mut parser = JsonParser::new("1.5e2");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(150.0)));
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
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
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
            '0'..='9' | '-' => self.parse_number(),
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
                    return Err("Unterminated escape sequence".to_string());
                }
                let next_char = self.input.chars().nth(self.pos).unwrap();
                match next_char {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(format!("Invalid escape sequence: \\{}", next_char)),
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
        let mut has_exponent = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            match c {
                '0'..='9' => {
                    self.pos += 1;
                }
                '.' => {
                    if has_dot || has_exponent {
                        return Err("Invalid number format".to_string());
                    }
                    has_dot = true;
                    self.pos += 1;
                }
                'e' | 'E' => {
                    if has_exponent {
                        return Err("Invalid number format".to_string());
                    }
                    has_exponent = true;
                    self.pos += 1;
                    if self.pos < self.input.len() {
                        let next_char = self.input.chars().nth(self.pos).unwrap();
                        if next_char == '+' || next_char == '-' {
                            self.pos += 1;
                        }
                    }
                }
                '+' | '-' => {
                    if self.pos == start {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
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
            if self.pos >= self.input.len() {
                return Err("Unterminated object".to_string());
            }

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
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));

        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));

        let mut parser = JsonParser::new("1.23e-4");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(1.23e-4)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
        );

        let mut parser = JsonParser::new(r#""escape\"test""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("escape\"test".to_string()))
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
                JsonValue::Number(3.0)
            ]))
        );
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value", "num": 42}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        expected.insert("num".to_string(), JsonValue::Number(42.0));
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
        let mut escaped = false;
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if escaped {
                match c {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(format!("Invalid escape sequence: \\{}", c)),
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                self.pos += 1;
                return Ok(JsonValue::String(result));
            } else {
                result.push(c);
            }
            self.pos += 1;
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;
        let mut has_exp = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            match c {
                '0'..='9' => {}
                '.' if !has_dot && !has_exp => has_dot = true,
                'e' | 'E' if !has_exp => {
                    has_exp = true;
                    if self.pos + 1 < self.input.len() {
                        let next = self.input.chars().nth(self.pos + 1).unwrap();
                        if next == '+' || next == '-' {
                            self.pos += 1;
                        }
                    }
                }
                _ => break,
            }
            self.pos += 1;
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '['
        let mut array = Vec::new();
        self.skip_whitespace();

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
        let mut object = HashMap::new();
        self.skip_whitespace();

        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
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
            return Err("Extra characters after JSON value".to_string());
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

        let mut parser = JsonParser::new("1.23e-4");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(1.23e-4)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
        );

        let mut parser = JsonParser::new(r#""escape\nsequence""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("escape\nsequence".to_string()))
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
use std::error::Error;

#[derive(Debug, PartialEq)]
enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

struct JsonParser {
    input: String,
    position: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            input: input.to_string(),
            position: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err("Unexpected trailing characters".into());
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('"') => self.parse_string(),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('t') | Some('f') => self.parse_boolean(),
            Some('n') => self.parse_null(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err("Invalid JSON value".into()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        self.consume_char('"')?;
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.next_char().ok_or("Unexpected end of string")?;
                    result.push(self.parse_escape(escaped)?);
                }
                _ => result.push(c),
            }
        }
        Ok(JsonValue::String(result))
    }

    fn parse_escape(&self, c: char) -> Result<char, Box<dyn Error>> {
        match c {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'b' => Ok('\u{0008}'),
            'f' => Ok('\u{000C}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            _ => Err("Invalid escape sequence".into()),
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        let start = self.position;
        let mut has_decimal = false;
        let mut has_exponent = false;

        if self.peek_char() == Some('-') {
            self.next_char();
        }

        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.next_char();
            } else if c == '.' && !has_decimal && !has_exponent {
                has_decimal = true;
                self.next_char();
            } else if (c == 'e' || c == 'E') && !has_exponent {
                has_exponent = true;
                self.next_char();
                if self.peek_char() == Some('-') || self.peek_char() == Some('+') {
                    self.next_char();
                }
            } else {
                break;
            }
        }

        let num_str = &self.input[start..self.position];
        num_str
            .parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|e| e.into())
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        if self.input[self.position..].starts_with("true") {
            self.position += 4;
            Ok(JsonValue::Boolean(true))
        } else if self.input[self.position..].starts_with("false") {
            self.position += 5;
            Ok(JsonValue::Boolean(false))
        } else {
            Err("Invalid boolean value".into())
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        if self.input[self.position..].starts_with("null") {
            self.position += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Invalid null value".into())
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        self.consume_char('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();

        if self.peek_char() != Some('}') {
            loop {
                self.skip_whitespace();
                let key = match self.parse_string()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Object key must be string".into()),
                };
                self.skip_whitespace();
                self.consume_char(':')?;
                let value = self.parse_value()?;
                map.insert(key, value);
                self.skip_whitespace();
                if self.peek_char() != Some(',') {
                    break;
                }
                self.next_char();
            }
        }
        self.consume_char('}')?;
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, Box<dyn Error>> {
        self.consume_char('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.peek_char() != Some(']') {
            loop {
                let value = self.parse_value()?;
                array.push(value);
                self.skip_whitespace();
                if self.peek_char() != Some(',') {
                    break;
                }
                self.next_char();
            }
        }
        self.consume_char(']')?;
        Ok(JsonValue::Array(array))
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
        self.input.chars().nth(self.position)
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn consume_char(&mut self, expected: char) -> Result<(), Box<dyn Error>> {
        match self.next_char() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(format!("Expected '{}', found '{}'", expected, c).into()),
            None => Err("Unexpected end of input".into()),
        }
    }
}

fn extract_values(json: &str, keys: &[&str]) -> Result<Vec<Option<String>>, Box<dyn Error>> {
    let mut parser = JsonParser::new(json);
    let value = parser.parse()?;
    
    let mut results = Vec::new();
    if let JsonValue::Object(map) = value {
        for key in keys {
            match map.get(*key) {
                Some(JsonValue::String(s)) => results.push(Some(s.clone())),
                Some(JsonValue::Number(n)) => results.push(Some(n.to_string())),
                Some(JsonValue::Boolean(b)) => results.push(Some(b.to_string())),
                Some(JsonValue::Null) => results.push(Some("null".to_string())),
                Some(_) => results.push(None),
                None => results.push(None),
            }
        }
    } else {
        return Err("Expected JSON object".into());
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let json = r#"{"name": "John", "age": 30, "active": true}"#;
        let mut parser = JsonParser::new(json);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_value_extraction() {
        let json = r#"{"name": "Alice", "score": 95.5, "passed": true}"#;
        let keys = vec!["name", "score", "passed", "missing"];
        let result = extract_values(json, &keys).unwrap();
        assert_eq!(result, vec![
            Some("Alice".to_string()),
            Some("95.5".to_string()),
            Some("true".to_string()),
            None
        ]);
    }
}