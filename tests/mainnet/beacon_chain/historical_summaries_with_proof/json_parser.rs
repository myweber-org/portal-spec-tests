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
                    return Err("Unexpected end of string".to_string());
                }
                let escaped = self.input.chars().nth(self.pos).unwrap();
                match escaped {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
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

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '.' {
                if has_dot {
                    return Err("Invalid number format".to_string());
                }
                has_dot = true;
                self.pos += 1;
            } else if c.is_digit(10) {
                self.pos += 1;
            } else {
                break;
            }
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
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

        let mut parser = JsonParser::new("3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(3.14)));
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