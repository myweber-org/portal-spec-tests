use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
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
            Some('"') => self.parse_string(),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('t') | Some('f') => self.parse_boolean(),
            Some('n') => self.parse_null(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            Some(c) => Err(ParseError {
                message: format!("Unexpected character: {}", c),
                position: self.position,
            }),
            None => Err(ParseError {
                message: "Unexpected end of input".to_string(),
                position: self.position,
            }),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('"')?;
        let mut result = String::new();
        
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.next_char().ok_or_else(|| ParseError {
                        message: "Unterminated escape sequence".to_string(),
                        position: self.position,
                    })?;
                    result.push(self.parse_escape_char(escaped)?);
                }
                _ => result.push(c),
            }
        }
        
        Ok(JsonValue::String(result))
    }

    fn parse_escape_char(&self, c: char) -> Result<char, ParseError> {
        match c {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'b' => Ok('\x08'),
            'f' => Ok('\x0c'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            _ => Err(ParseError {
                message: format!("Invalid escape character: {}", c),
                position: self.position,
            }),
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_decimal = false;
        let mut has_exponent = false;
        
        if self.peek_char() == Some('-') {
            self.next_char();
        }
        
        while let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => {
                    self.next_char();
                }
                '.' if !has_decimal && !has_exponent => {
                    has_decimal = true;
                    self.next_char();
                }
                'e' | 'E' if !has_exponent => {
                    has_exponent = true;
                    self.next_char();
                    if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                        self.next_char();
                    }
                }
                _ => break,
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError {
                message: format!("Invalid number: {}", number_str),
                position: start,
            }),
        }
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("true") {
            Ok(JsonValue::Boolean(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Boolean(false))
        } else {
            Err(ParseError {
                message: "Expected boolean value".to_string(),
                position: self.position,
            })
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err(ParseError {
                message: "Expected null value".to_string(),
                position: self.position,
            })
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('{')?;
        self.skip_whitespace();
        
        let mut map = HashMap::new();
        
        if self.peek_char() == Some('}') {
            self.next_char();
            return Ok(JsonValue::Object(map));
        }
        
        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            self.consume_char(':')?;
            self.skip_whitespace();
            
            let value = self.parse_value()?;
            map.insert(key, value);
            
            self.skip_whitespace();
            if self.peek_char() == Some('}') {
                self.next_char();
                break;
            }
            
            self.consume_char(',')?;
        }
        
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_char('[')?;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek_char() == Some(']') {
            self.next_char();
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                self.next_char();
                break;
            }
            
            self.consume_char(',')?;
        }
        
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
        self.input.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn consume_char(&mut self, expected: char) -> Result<(), ParseError> {
        match self.next_char() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(ParseError {
                message: format!("Expected '{}', found '{}'", expected, c),
                position: self.position - 1,
            }),
            None => Err(ParseError {
                message: format!("Expected '{}', found end of input", expected),
                position: self.position,
            }),
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        let expected_chars: Vec<char> = expected.chars().collect();
        if self.position + expected_chars.len() > self.input.len() {
            return false;
        }
        
        for (i, &c) in expected_chars.iter().enumerate() {
            if self.input[self.position + i] != c {
                return false;
            }
        }
        
        self.position += expected_chars.len();
        true
    }
}

pub fn parse_json(json_str: &str) -> Result<HashMap<String, JsonValue>, Box<dyn Error>> {
    let mut parser = JsonParser::new(json_str);
    match parser.parse()? {
        JsonValue::Object(map) => Ok(map),
        _ => Err("Top-level JSON value must be an object".into()),
    }
}