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
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for ch in expected.chars() {
            self.consume(ch)?;
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        let next = self.peek().unwrap();
        if next == 't' {
            let expected = "true";
            for ch in expected.chars() {
                self.consume(ch)?;
            }
            Ok(JsonValue::Bool(true))
        } else {
            let expected = "false";
            for ch in expected.chars() {
                self.consume(ch)?;
            }
            Ok(JsonValue::Bool(false))
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

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if self.peek() == Some(']') {
            self.consume(']')?;
            return Ok(JsonValue::Array(array));
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if self.peek() == Some(']') {
                self.consume(']')?;
                break;
            }
            self.consume(',')?;
            self.skip_whitespace();
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek() == Some('}') {
            self.consume('}')?;
            return Ok(JsonValue::Object(map));
        }
        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };
            self.skip_whitespace();
            self.consume(':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if self.peek() == Some('}') {
                self.consume('}')?;
                break;
            }
            self.consume(',')?;
            self.skip_whitespace();
        }
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
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello".to_string())));
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
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedCharacter(char, usize),
    UnexpectedEnd,
    InvalidNumber,
    InvalidEscapeSequence,
    TrailingComma,
    ExpectedColon,
    ExpectedKey,
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
            Some(ch) => Err(ParseError::UnexpectedCharacter(ch, self.position)),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.consume('"')?;
        let mut result = String::new();
        let mut escape = false;

        while let Some(ch) = self.peek() {
            self.position += 1;

            if escape {
                match ch {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\u{0008}'),
                    'f' => result.push('\u{000C}'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    'u' => return Err(ParseError::InvalidEscapeSequence),
                    _ => return Err(ParseError::InvalidEscapeSequence),
                }
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                return Ok(result);
            } else {
                result.push(ch);
            }
        }

        Err(ParseError::UnexpectedEnd)
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.position;
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(ch) = self.peek() {
            match ch {
                '0'..='9' => {
                    self.position += 1;
                }
                '.' => {
                    if has_dot || has_exp {
                        return Err(ParseError::InvalidNumber);
                    }
                    has_dot = true;
                    self.position += 1;
                }
                'e' | 'E' => {
                    if has_exp {
                        return Err(ParseError::InvalidNumber);
                    }
                    has_exp = true;
                    self.position += 1;
                    if let Some(next) = self.peek() {
                        if next == '+' || next == '-' {
                            self.position += 1;
                        }
                    }
                }
                _ => break,
            }
        }

        let slice: String = self.input[start..self.position].iter().collect();
        slice.parse().map_err(|_| ParseError::InvalidNumber)
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, ParseError> {
        self.consume('[')?;
        self.skip_whitespace();

        if let Some(']') = self.peek() {
            self.position += 1;
            return Ok(Vec::new());
        }

        let mut items = Vec::new();
        loop {
            items.push(self.parse_value()?);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.position += 1;
                    self.skip_whitespace();
                    if let Some(']') = self.peek() {
                        return Err(ParseError::TrailingComma);
                    }
                }
                Some(']') => {
                    self.position += 1;
                    break;
                }
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position)),
                None => return Err(ParseError::UnexpectedEnd),
            }
        }

        Ok(items)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, ParseError> {
        self.consume('{')?;
        self.skip_whitespace();

        if let Some('}') = self.peek() {
            self.position += 1;
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();

            self.consume(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.position += 1;
                    self.skip_whitespace();
                    if let Some('}') = self.peek() {
                        return Err(ParseError::TrailingComma);
                    }
                }
                Some('}') => {
                    self.position += 1;
                    break;
                }
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position)),
                None => return Err(ParseError::UnexpectedEnd),
            }
        }

        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some('n') => {
                if self.input[self.position..].starts_with(&['n', 'u', 'l', 'l']) {
                    self.position += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err(ParseError::UnexpectedCharacter('n', self.position))
                }
            }
            Some('t') => {
                if self.input[self.position..].starts_with(&['t', 'r', 'u', 'e']) {
                    self.position += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err(ParseError::UnexpectedCharacter('t', self.position))
                }
            }
            Some('f') => {
                if self.input[self.position..].starts_with(&['f', 'a', 'l', 's', 'e']) {
                    self.position += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err(ParseError::UnexpectedCharacter('f', self.position))
                }
            }
            Some('"') => {
                let s = self.parse_string()?;
                Ok(JsonValue::String(s))
            }
            Some('[') => {
                let arr = self.parse_array()?;
                Ok(JsonValue::Array(arr))
            }
            Some('{') => {
                let obj = self.parse_object()?;
                Ok(JsonValue::Object(obj))
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                let num = self.parse_number()?;
                Ok(JsonValue::Number(num))
            }
            Some(ch) => Err(ParseError::UnexpectedCharacter(ch, self.position)),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ))
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

        let mut parser = JsonParser::new("1.5e2");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(150.0)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello".to_string()))
        );

        let mut parser = JsonParser::new(r#""test\nvalue""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("test\nvalue".to_string()))
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

    #[test]
    fn test_parse_error() {
        let mut parser = JsonParser::new("[1, 2,]");
        assert_eq!(parser.parse(), Err(ParseError::TrailingComma));

        let mut parser = JsonParser::new("{");
        assert_eq!(parser.parse(), Err(ParseError::UnexpectedEnd));
    }
}