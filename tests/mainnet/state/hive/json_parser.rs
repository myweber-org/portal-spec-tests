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

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        match self.peek() {
            Some(ch) if ch == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(ch) => Err(format!("Expected '{}', found '{}'", expected, ch)),
            None => Err(format!("Expected '{}', found EOF", expected)),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('n') => self.parse_null(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err("Invalid JSON".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        let mut map = HashMap::new();

        self.skip_whitespace();
        if let Some('}') = self.peek() {
            self.pos += 1;
            return Ok(JsonValue::Object(map));
        }

        loop {
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };

            self.consume(':')?;
            let value = self.parse()?;
            map.insert(key, value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err("Expected ',' or '}'".to_string()),
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        let mut arr = Vec::new();

        self.skip_whitespace();
        if let Some(']') = self.peek() {
            self.pos += 1;
            return Ok(JsonValue::Array(arr));
        }

        loop {
            let value = self.parse()?;
            arr.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err("Expected ',' or ']'".to_string()),
            }
        }

        Ok(JsonValue::Array(arr))
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume('"')?;
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.pos += 1;
                return Ok(JsonValue::String(result));
            } else if ch == '\\' {
                self.pos += 1;
                match self.peek() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\x08'),
                    Some('f') => result.push('\x0c'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(_) => return Err("Invalid escape sequence".to_string()),
                    None => return Err("Unexpected EOF".to_string()),
                }
                self.pos += 1;
            } else {
                result.push(ch);
                self.pos += 1;
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;

        if let Some('-') = self.peek() {
            self.pos += 1;
        }

        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                self.pos += 1;
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.pos += 1;
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

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Invalid boolean".to_string())
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with(&['n', 'u', 'l', 'l']) {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Invalid null".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42.5");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.5)));
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(true)));
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
}