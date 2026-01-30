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
            Err("Expected boolean value".to_string())
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
        
        if self.input.chars().nth(self.pos) == Some('-') {
            self.pos += 1;
        }

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }

        if self.pos < self.input.len() && self.input.chars().nth(self.pos) == Some('.') {
            self.pos += 1;
            while self.pos < self.input.len() {
                let c = self.input.chars().nth(self.pos).unwrap();
                if c.is_ascii_digit() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }

        let number_str = &self.input[start..self.pos];
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", number_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '['
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.pos < self.input.len() && self.input.chars().nth(self.pos) == Some(']') {
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
        
        if self.pos < self.input.len() && self.input.chars().nth(self.pos) == Some('}') {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            
            if self.pos >= self.input.len() {
                return Err("Unterminated object".to_string());
            }
            
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };
            
            self.skip_whitespace();
            
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos) != Some(':') {
                return Err("Expected ':' after object key".to_string());
            }
            
            self.pos += 1; // Skip ':'
            
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
                return Err(format!("Expected ',' or '}', found: {}", c));
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
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new("\"hello world\"");
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
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
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    Null,
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
            _ => Err(ParseError {
                message: "Unexpected character".to_string(),
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
                    let escaped = self.next_char().ok_or(ParseError {
                        message: "Unterminated escape sequence".to_string(),
                        position: self.position,
                    })?;
                    result.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'b' => '\x08',
                        'f' => '\x0c',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        _ => return Err(ParseError {
                            message: format!("Invalid escape sequence: \\{}", escaped),
                            position: self.position - 2,
                        }),
                    });
                }
                _ => result.push(c),
            }
        }
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
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
                if !self.peek_char().map_or(false, |c| c.is_digit(10)) {
                    return Err(ParseError {
                        message: "Expected digit after decimal point".to_string(),
                        position: self.position,
                    });
                }
            } else if (c == 'e' || c == 'E') && !has_exponent {
                has_exponent = true;
                self.next_char();
                if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                    self.next_char();
                }
                if !self.peek_char().map_or(false, |c| c.is_digit(10)) {
                    return Err(ParseError {
                        message: "Expected digit in exponent".to_string(),
                        position: self.position,
                    });
                }
            } else {
                break;
            }
        }

        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError {
                message: "Invalid number format".to_string(),
                position: start,
            }),
        }
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("true").is_ok() {
            Ok(JsonValue::Boolean(true))
        } else if self.consume_str("false").is_ok() {
            Ok(JsonValue::Boolean(false))
        } else {
            Err(ParseError {
                message: "Expected boolean value".to_string(),
                position: self.position,
            })
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        self.consume_str("null")?;
        Ok(JsonValue::Null)
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
                message: format!("Expected '{}', found EOF", expected),
                position: self.position,
            }),
        }
    }

    fn consume_str(&mut self, expected: &str) -> Result<(), ParseError> {
        for (i, expected_char) in expected.chars().enumerate() {
            if self.position + i >= self.input.len() {
                return Err(ParseError {
                    message: format!("Expected '{}', found EOF", expected),
                    position: self.position + i,
                });
            }
            if self.input[self.position + i] != expected_char {
                return Err(ParseError {
                    message: format!("Expected '{}'", expected),
                    position: self.position + i,
                });
            }
        }
        self.position += expected.len();
        Ok(())
    }
}

fn extract_values(json: &JsonValue, key: &str) -> Vec<&JsonValue> {
    let mut results = Vec::new();
    match json {
        JsonValue::Object(map) => {
            if let Some(value) = map.get(key) {
                results.push(value);
            }
            for value in map.values() {
                results.extend(extract_values(value, key));
            }
        }
        JsonValue::Array(arr) => {
            for value in arr {
                results.extend(extract_values(value, key));
            }
        }
        _ => {}
    }
    results
}

fn main() {
    let json_str = r#"
    {
        "name": "test",
        "values": [1, 2, 3],
        "nested": {
            "name": "inner",
            "flag": true
        }
    }"#;

    match JsonParser::new(json_str).parse() {
        Ok(json) => {
            println!("Parsed JSON successfully");
            let names = extract_values(&json, "name");
            for name in names {
                if let JsonValue::String(s) = name {
                    println!("Found name: {}", s);
                }
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}