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
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl Error for ParseError {}

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

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(self.error("Unexpected trailing characters"));
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
            _ => Err(self.error("Expected a JSON value")),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("null")?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_if("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_if("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(self.error("Expected 'true' or 'false'"))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.pos;
        if self.consume_if("-") {
            // just consume the minus sign
        }
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                self.consume_char();
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(self.error(&format!("Invalid number format: {}", num_str))),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("\"")?;
        let mut result = String::new();
        while let Some(c) = self.consume_char() {
            match c {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    let escaped = self.consume_char().ok_or_else(|| self.error("Unterminated escape sequence"))?;
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\u{0008}'),
                        'f' => result.push('\u{000C}'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        'u' => {
                            let hex_code: String = (0..4)
                                .map(|_| self.consume_char().ok_or_else(|| self.error("Incomplete Unicode escape")))
                                .collect::<Result<_, _>>()?;
                            let code_point = u32::from_str_radix(&hex_code, 16)
                                .map_err(|_| self.error("Invalid Unicode escape"))?;
                            let ch = char::from_u32(code_point).ok_or_else(|| self.error("Invalid Unicode code point"))?;
                            result.push(ch);
                        }
                        _ => return Err(self.error("Invalid escape character")),
                    }
                }
                _ => result.push(c),
            }
        }
        Err(self.error("Unterminated string"))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("[")?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if self.consume_if("]") {
            return Ok(JsonValue::Array(array));
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if self.consume_if("]") {
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
        if self.consume_if("}") {
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
            if self.consume_if("}") {
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
                self.consume_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume_char(&mut self) -> Option<char> {
        let c = self.input.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn consume_if(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.pos + chars.len() <= self.input.len() {
            let slice = &self.input[self.pos..self.pos + chars.len()];
            if slice == chars.as_slice() {
                self.pos += chars.len();
                return true;
            }
        }
        false
    }

    fn expect(&mut self, s: &str) -> Result<(), ParseError> {
        if self.consume_if(s) {
            Ok(())
        } else {
            Err(self.error(&format!("Expected '{}'", s)))
        }
    }

    fn error(&self, msg: &str) -> ParseError {
        ParseError {
            message: msg.to_string(),
            position: self.pos,
        }
    }
}

pub fn parse_json(json_str: &str) -> Result<JsonValue, ParseError> {
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
        assert_eq!(parse_json("1.23e4").unwrap(), JsonValue::Number(12300.0));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_json(r#""hello""#).unwrap(), JsonValue::String("hello".to_string()));
        assert_eq!(parse_json(r#""escape\"test""#).unwrap(), JsonValue::String("escape\"test".to_string()));
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
        let result = parse_json(r#"{"key": "value", "num": 42}"#).unwrap();
        if let JsonValue::Object(map) = result {
            assert_eq!(map.get("key"), Some(&JsonValue::String("value".to_string())));
            assert_eq!(map.get("num"), Some(&JsonValue::Number(42.0)));
        } else {
            panic!("Expected object");
        }
    }
}