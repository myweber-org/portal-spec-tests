use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

fn parse_json_string(input: &str) -> Result<(JsonValue, &str), &'static str> {
    let mut chars = input.chars();
    match chars.next() {
        Some('"') => {
            let mut result = String::new();
            let mut escaped = false;
            let mut remaining = &input[1..];
            
            for (i, ch) in remaining.char_indices() {
                if escaped {
                    match ch {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        '/' => result.push('/'),
                        _ => return Err("Invalid escape sequence"),
                    }
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    return Ok((JsonValue::String(result), &remaining[i+1..]));
                } else {
                    result.push(ch);
                }
            }
            Err("Unterminated string")
        }
        _ => Err("Expected string"),
    }
}

fn parse_json_number(input: &str) -> Result<(JsonValue, &str), &'static str> {
    let mut end = 0;
    let chars: Vec<char> = input.chars().collect();
    
    if chars.is_empty() {
        return Err("Empty input for number");
    }
    
    if chars[0] == '-' {
        end += 1;
    }
    
    while end < chars.len() && chars[end].is_ascii_digit() {
        end += 1;
    }
    
    if end < chars.len() && chars[end] == '.' {
        end += 1;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
    }
    
    if end < chars.len() && (chars[end] == 'e' || chars[end] == 'E') {
        end += 1;
        if end < chars.len() && (chars[end] == '+' || chars[end] == '-') {
            end += 1;
        }
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
    }
    
    if end == 0 {
        return Err("Invalid number format");
    }
    
    let num_str = &input[..end];
    match num_str.parse::<f64>() {
        Ok(num) => Ok((JsonValue::Number(num), &input[end..])),
        Err(_) => Err("Failed to parse number"),
    }
}

fn parse_json_value(input: &str) -> Result<(JsonValue, &str), &'static str> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return Err("Empty input");
    }
    
    match trimmed.chars().next().unwrap() {
        'n' => {
            if trimmed.starts_with("null") {
                Ok((JsonValue::Null, &trimmed[4..]))
            } else {
                Err("Expected null")
            }
        }
        't' => {
            if trimmed.starts_with("true") {
                Ok((JsonValue::Bool(true), &trimmed[4..]))
            } else {
                Err("Expected true")
            }
        }
        'f' => {
            if trimmed.starts_with("false") {
                Ok((JsonValue::Bool(false), &trimmed[5..]))
            } else {
                Err("Expected false")
            }
        }
        '"' => parse_json_string(trimmed),
        '[' => parse_json_array(trimmed),
        '{' => parse_json_object(trimmed),
        '-' | '0'..='9' => parse_json_number(trimmed),
        _ => Err("Unexpected character"),
    }
}

fn parse_json_array(input: &str) -> Result<(JsonValue, &str), &'static str> {
    let mut chars = input.chars();
    if chars.next() != Some('[') {
        return Err("Expected '[' for array");
    }
    
    let mut remaining = &input[1..];
    let mut array = Vec::new();
    
    loop {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            return Err("Unterminated array");
        }
        
        if remaining.starts_with(']') {
            return Ok((JsonValue::Array(array), &remaining[1..]));
        }
        
        let (value, next) = parse_json_value(remaining)?;
        array.push(value);
        remaining = next.trim_start();
        
        if remaining.is_empty() {
            return Err("Unterminated array");
        }
        
        match remaining.chars().next().unwrap() {
            ',' => {
                remaining = &remaining[1..];
                continue;
            }
            ']' => {
                return Ok((JsonValue::Array(array), &remaining[1..]));
            }
            _ => return Err("Expected ',' or ']' in array"),
        }
    }
}

fn parse_json_object(input: &str) -> Result<(JsonValue, &str), &'static str> {
    let mut chars = input.chars();
    if chars.next() != Some('{') {
        return Err("Expected '{' for object");
    }
    
    let mut remaining = &input[1..];
    let mut object = HashMap::new();
    
    loop {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            return Err("Unterminated object");
        }
        
        if remaining.starts_with('}') {
            return Ok((JsonValue::Object(object), &remaining[1..]));
        }
        
        let (key, next) = parse_json_string(remaining)?;
        let key = match key {
            JsonValue::String(s) => s,
            _ => return Err("Expected string key"),
        };
        
        remaining = next.trim_start();
        if remaining.is_empty() || !remaining.starts_with(':') {
            return Err("Expected ':' after object key");
        }
        
        remaining = &remaining[1..].trim_start();
        let (value, next) = parse_json_value(remaining)?;
        object.insert(key, value);
        remaining = next.trim_start();
        
        if remaining.is_empty() {
            return Err("Unterminated object");
        }
        
        match remaining.chars().next().unwrap() {
            ',' => {
                remaining = &remaining[1..];
                continue;
            }
            '}' => {
                return Ok((JsonValue::Object(object), &remaining[1..]));
            }
            _ => return Err("Expected ',' or '}' in object"),
        }
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, &'static str> {
    let (value, remaining) = parse_json_value(input.trim())?;
    if !remaining.trim().is_empty() {
        return Err("Trailing characters after JSON value");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_values() {
        assert_eq!(parse_json("null"), Ok(JsonValue::Null));
        assert_eq!(parse_json("true"), Ok(JsonValue::Bool(true)));
        assert_eq!(parse_json("false"), Ok(JsonValue::Bool(false)));
        assert_eq!(parse_json("\"hello\""), Ok(JsonValue::String("hello".to_string())));
        assert_eq!(parse_json("42"), Ok(JsonValue::Number(42.0)));
        assert_eq!(parse_json("-3.14"), Ok(JsonValue::Number(-3.14)));
    }
    
    #[test]
    fn test_parse_array() {
        let result = parse_json("[1, 2, 3]");
        assert!(result.is_ok());
        if let Ok(JsonValue::Array(arr)) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::Number(2.0));
            assert_eq!(arr[2], JsonValue::Number(3.0));
        }
    }
    
    #[test]
    fn test_parse_object() {
        let result = parse_json(r#"{"name": "John", "age": 30}"#);
        assert!(result.is_ok());
        if let Ok(JsonValue::Object(obj)) = result {
            assert_eq!(obj.len(), 2);
            assert_eq!(obj.get("name"), Some(&JsonValue::String("John".to_string())));
            assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
        }
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

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

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.current_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume_char('"');
        let mut result = String::new();
        while let Some(c) = self.current_char() {
            if c == '"' {
                break;
            }
            if c == '\\' {
                self.advance();
                match self.current_char() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\x08'),
                    Some('f') => result.push('\x0c'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(_) => return Err("Invalid escape sequence".to_string()),
                    None => return Err("Unexpected end of string".to_string()),
                }
            } else {
                result.push(c);
            }
            self.advance();
        }
        if self.consume_char('"') {
            Ok(JsonValue::String(result))
        } else {
            Err("Unterminated string".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        if self.current_char() == Some('-') {
            self.advance();
        }
        while let Some(c) = self.current_char() {
            if !c.is_digit(10) {
                break;
            }
            self.advance();
        }
        if self.current_char() == Some('.') {
            self.advance();
            while let Some(c) = self.current_char() {
                if !c.is_digit(10) {
                    break;
                }
                self.advance();
            }
        }
        if let Some('e') | Some('E') = self.current_char() {
            self.advance();
            if let Some('+') | Some('-') = self.current_char() {
                self.advance();
            }
            while let Some(c) = self.current_char() {
                if !c.is_digit(10) {
                    break;
                }
                self.advance();
            }
        }
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume_char('[');
        self.skip_whitespace();
        let mut array = Vec::new();
        if self.current_char() == Some(']') {
            self.advance();
            return Ok(JsonValue::Array(array));
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if self.current_char() == Some(']') {
                self.advance();
                break;
            }
            if !self.consume_char(',') {
                return Err("Expected ',' or ']' in array".to_string());
            }
            self.skip_whitespace();
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume_char('{');
        self.skip_whitespace();
        let mut object = HashMap::new();
        if self.current_char() == Some('}') {
            self.advance();
            return Ok(JsonValue::Object(object));
        }
        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };
            self.skip_whitespace();
            if !self.consume_char(':') {
                return Err("Expected ':' after object key".to_string());
            }
            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();
            if self.current_char() == Some('}') {
                self.advance();
                break;
            }
            if !self.consume_char(',') {
                return Err("Expected ',' or '}' in object".to_string());
            }
            self.skip_whitespace();
        }
        Ok(JsonValue::Object(object))
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.current_char() == Some(expected) {
            self.advance();
            true
        } else {
            false
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

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }
}

pub fn parse_json(json_str: &str) -> Result<JsonValue, String> {
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
        assert_eq!(parse_json("1.5e2").unwrap(), JsonValue::Number(150.0));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_json(r#""hello""#).unwrap(),
            JsonValue::String("hello".to_string())
        );
        assert_eq!(
            parse_json(r#""escape\nnewline""#).unwrap(),
            JsonValue::String("escape\nnewline".to_string())
        );
    }

    #[test]
    fn test_parse_array() {
        let result = parse_json("[1, true, null]").unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::Bool(true));
            assert_eq!(arr[2], JsonValue::Null);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_object() {
        let result = parse_json(r#"{"key": "value", "num": 42}"#).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 2);
            assert_eq!(
                obj.get("key").unwrap(),
                &JsonValue::String("value".to_string())
            );
            assert_eq!(obj.get("num").unwrap(), &JsonValue::Number(42.0));
        } else {
            panic!("Expected object");
        }
    }
}