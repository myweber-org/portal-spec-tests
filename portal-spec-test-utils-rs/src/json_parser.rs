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
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(ParseError::UnexpectedEndOfInput),
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
            Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ))
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.advance(1); // Skip opening quote
        let mut result = String::new();

        while let Some(c) = self.peek_char() {
            match c {
                '"' => {
                    self.advance(1);
                    return Ok(JsonValue::String(result));
                }
                '\\' => {
                    self.advance(1);
                    let escaped = self.parse_escape_sequence()?;
                    result.push(escaped);
                }
                _ => {
                    result.push(c);
                    self.advance(1);
                }
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_escape_sequence(&mut self) -> Result<char, ParseError> {
        match self.peek_char() {
            Some('"') => {
                self.advance(1);
                Ok('"')
            }
            Some('\\') => {
                self.advance(1);
                Ok('\\')
            }
            Some('/') => {
                self.advance(1);
                Ok('/')
            }
            Some('b') => {
                self.advance(1);
                Ok('\x08')
            }
            Some('f') => {
                self.advance(1);
                Ok('\x0C')
            }
            Some('n') => {
                self.advance(1);
                Ok('\n')
            }
            Some('r') => {
                self.advance(1);
                Ok('\r')
            }
            Some('t') => {
                self.advance(1);
                Ok('\t')
            }
            _ => Err(ParseError::InvalidEscapeSequence),
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_decimal = false;
        let mut has_exponent = false;

        if self.peek_char() == Some('-') {
            self.advance(1);
        }

        while let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => {
                    self.advance(1);
                }
                '.' => {
                    if has_decimal || has_exponent {
                        return Err(ParseError::InvalidNumber);
                    }
                    has_decimal = true;
                    self.advance(1);
                }
                'e' | 'E' => {
                    if has_exponent {
                        return Err(ParseError::InvalidNumber);
                    }
                    has_exponent = true;
                    self.advance(1);
                    if self.peek_char() == Some('-') || self.peek_char() == Some('+') {
                        self.advance(1);
                    }
                }
                _ => break,
            }
        }

        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError::InvalidNumber),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.advance(1); // Skip '['
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
                Some(c) => return Err(ParseError::UnexpectedCharacter(c, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.advance(1); // Skip '{'
        self.skip_whitespace();
        let mut object = HashMap::new();

        if self.peek_char() == Some('}') {
            self.advance(1);
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err(ParseError::KeyMustBeString),
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
                Some(c) => return Err(ParseError::UnexpectedCharacter(c, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
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

    fn advance(&mut self, n: usize) {
        self.position += n;
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
            self.advance(expected.len());
            Ok(())
        } else {
            Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ))
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
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
        );
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
}