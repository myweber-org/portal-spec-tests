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
    chars: Vec<char>,
    index: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            chars: input.chars().collect(),
            index: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.index < self.chars.len() && self.chars[self.index].is_whitespace() {
            self.index += 1;
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if self.index >= self.chars.len() {
            return Err("Unexpected end of input".to_string());
        }

        match self.chars[self.index] {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", self.chars[self.index])),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.index + 3 < self.chars.len()
            && self.chars[self.index..self.index + 4].iter().collect::<String>() == "null"
        {
            self.index += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.index + 3 < self.chars.len()
            && self.chars[self.index..self.index + 4].iter().collect::<String>() == "true"
        {
            self.index += 4;
            Ok(JsonValue::Bool(true))
        } else if self.index + 4 < self.chars.len()
            && self.chars[self.index..self.index + 5].iter().collect::<String>() == "false"
        {
            self.index += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.index += 1;
        let mut result = String::new();

        while self.index < self.chars.len() && self.chars[self.index] != '"' {
            result.push(self.chars[self.index]);
            self.index += 1;
        }

        if self.index < self.chars.len() && self.chars[self.index] == '"' {
            self.index += 1;
            Ok(JsonValue::String(result))
        } else {
            Err("Unterminated string".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.index;
        while self.index < self.chars.len()
            && (self.chars[self.index].is_ascii_digit()
                || self.chars[self.index] == '.'
                || self.chars[self.index] == '-'
                || self.chars[self.index] == 'e'
                || self.chars[self.index] == 'E'
                || self.chars[self.index] == '+')
        {
            self.index += 1;
        }

        let num_str: String = self.chars[start..self.index].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.index += 1;
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.index < self.chars.len() && self.chars[self.index] == ']' {
            self.index += 1;
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            if self.index < self.chars.len() && self.chars[self.index] == ',' {
                self.index += 1;
                self.skip_whitespace();
            } else if self.index < self.chars.len() && self.chars[self.index] == ']' {
                self.index += 1;
                break;
            } else {
                return Err("Expected ',' or ']' in array".to_string());
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.index += 1;
        self.skip_whitespace();
        let mut object = HashMap::new();

        if self.index < self.chars.len() && self.chars[self.index] == '}' {
            self.index += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            if self.index >= self.chars.len() || self.chars[self.index] != '"' {
                return Err("Expected string key in object".to_string());
            }

            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };

            self.skip_whitespace();
            if self.index >= self.chars.len() || self.chars[self.index] != ':' {
                return Err("Expected ':' after key in object".to_string());
            }
            self.index += 1;

            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();

            if self.index < self.chars.len() && self.chars[self.index] == ',' {
                self.index += 1;
                self.skip_whitespace();
            } else if self.index < self.chars.len() && self.chars[self.index] == '}' {
                self.index += 1;
                break;
            } else {
                return Err("Expected ',' or '}' in object".to_string());
            }
        }

        Ok(JsonValue::Object(object))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.index < self.chars.len() {
            return Err("Unexpected trailing characters".to_string());
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
        let mut parser = JsonParser::new(r#""hello""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello".to_string())));
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