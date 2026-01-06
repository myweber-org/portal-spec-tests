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

    fn consume(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
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

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.peek().is_some() {
            return Err("Unexpected trailing characters".to_string());
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
        for c in expected.chars() {
            if self.consume() != Some(c) {
                return Err(format!("Expected '{}'", expected));
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        let next = self.peek();
        if next == Some('t') {
            let expected = "true";
            for c in expected.chars() {
                if self.consume() != Some(c) {
                    return Err(format!("Expected '{}'", expected));
                }
            }
            Ok(JsonValue::Bool(true))
        } else if next == Some('f') {
            let expected = "false";
            for c in expected.chars() {
                if self.consume() != Some(c) {
                    return Err(format!("Expected '{}'", expected));
                }
            }
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected boolean".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;
        let mut has_exp = false;

        if self.peek() == Some('-') {
            self.consume();
        }

        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                self.consume();
            } else if ch == '.' && !has_dot && !has_exp {
                has_dot = true;
                self.consume();
            } else if (ch == 'e' || ch == 'E') && !has_exp {
                has_exp = true;
                self.consume();
                if self.peek() == Some('-') || self.peek() == Some('+') {
                    self.consume();
                }
            } else {
                break;
            }
        }

        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        if self.consume() != Some('"') {
            return Err("Expected '\"'".to_string());
        }

        let mut result = String::new();
        while let Some(ch) = self.consume() {
            match ch {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    if let Some(escaped) = self.parse_escape() {
                        result.push(escaped);
                    } else {
                        return Err("Invalid escape sequence".to_string());
                    }
                }
                _ => result.push(ch),
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_escape(&mut self) -> Option<char> {
        match self.consume()? {
            '"' => Some('"'),
            '\\' => Some('\\'),
            '/' => Some('/'),
            'b' => Some('\x08'),
            'f' => Some('\x0c'),
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            'u' => {
                let mut code = 0;
                for _ in 0..4 {
                    let digit = self.consume()?.to_digit(16)?;
                    code = code * 16 + digit;
                }
                std::char::from_u32(code)
            }
            _ => None,
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        if self.consume() != Some('[') {
            return Err("Expected '['".to_string());
        }

        let mut array = Vec::new();
        self.skip_whitespace();

        if self.peek() == Some(']') {
            self.consume();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.consume();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.consume();
                    break;
                }
                _ => return Err("Expected ',' or ']'".to_string()),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        if self.consume() != Some('{') {
            return Err("Expected '{'".to_string());
        }

        let mut map = HashMap::new();
        self.skip_whitespace();

        if self.peek() == Some('}') {
            self.consume();
            return Ok(JsonValue::Object(map));
        }

        loop {
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be string".to_string()),
            };

            self.skip_whitespace();
            if self.consume() != Some(':') {
                return Err("Expected ':'".to_string());
            }

            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.consume();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.consume();
                    break;
                }
                _ => return Err("Expected ',' or '}'".to_string()),
            }
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
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
    }
}