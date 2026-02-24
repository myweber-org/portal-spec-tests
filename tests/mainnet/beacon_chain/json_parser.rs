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
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn consume(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.position += 1;
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
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err(format!("Unexpected character at position {}", self.position)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for expected_char in expected.chars() {
            match self.consume() {
                Some(ch) if ch == expected_char => continue,
                _ => return Err(format!("Expected '{}'", expected)),
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        let first = self.consume().unwrap();
        if first == 't' {
            let expected = "rue";
            for expected_char in expected.chars() {
                match self.consume() {
                    Some(ch) if ch == expected_char => continue,
                    _ => return Err("Expected 'true'".to_string()),
                }
            }
            Ok(JsonValue::Bool(true))
        } else {
            let expected = "alse";
            for expected_char in expected.chars() {
                match self.consume() {
                    Some(ch) if ch == expected_char => continue,
                    _ => return Err("Expected 'false'".to_string()),
                }
            }
            Ok(JsonValue::Bool(false))
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume(); // Consume opening quote
        let mut result = String::new();
        
        while let Some(ch) = self.consume() {
            if ch == '"' {
                return Ok(JsonValue::String(result));
            }
            result.push(ch);
        }
        
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut num_str = String::new();
        
        if let Some('-') = self.peek() {
            num_str.push(self.consume().unwrap());
        }
        
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-' {
                num_str.push(self.consume().unwrap());
            } else {
                break;
            }
        }
        
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume(); // Consume '['
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if let Some(']') = self.peek() {
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
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume(); // Consume '{'
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if let Some('}') = self.peek() {
            self.consume();
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            
            if let Some('"') = self.peek() {
                let key = match self.parse_string()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Expected string key".to_string()),
                };
                
                self.skip_whitespace();
                match self.consume() {
                    Some(':') => (),
                    _ => return Err("Expected ':' after object key".to_string()),
                }
                
                self.skip_whitespace();
                let value = self.parse_value()?;
                object.insert(key, value);
                
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
                    _ => return Err("Expected ',' or '}' in object".to_string()),
                }
            } else {
                return Err("Expected string key in object".to_string());
            }
        }
        
        Ok(JsonValue::Object(object))
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
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
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
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse(), Ok(expected));
    }
}use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

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
pub struct JsonParser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> JsonParser<'a> {
    pub fn new(input: &'a str) -> Self {
        JsonParser {
            chars: input.chars().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        self.parse_value()
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.chars.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || *c == '-' => self.parse_number(),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for ch in expected.chars() {
            match self.chars.next() {
                Some(c) if c == ch => continue,
                _ => return Err("Expected 'null'".to_string()),
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        let mut buffer = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() {
                buffer.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        match buffer.as_str() {
            "true" => Ok(JsonValue::Bool(true)),
            "false" => Ok(JsonValue::Bool(false)),
            _ => Err("Invalid boolean value".to_string()),
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut buffer = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_digit(10) || *c == '-' || *c == '.' || *c == 'e' || *c == 'E' || *c == '+' {
                buffer.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        match buffer.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        let mut result = String::new();
        match self.chars.next() {
            Some('"') => {}
            _ => return Err("Expected opening quote".to_string()),
        }

        while let Some(c) = self.chars.next() {
            match c {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    if let Some(escaped) = self.chars.next() {
                        match escaped {
                            '"' => result.push('"'),
                            '\\' => result.push('\\'),
                            '/' => result.push('/'),
                            'b' => result.push('\x08'),
                            'f' => result.push('\x0c'),
                            'n' => result.push('\n'),
                            'r' => result.push('\r'),
                            't' => result.push('\t'),
                            _ => return Err("Invalid escape sequence".to_string()),
                        }
                    } else {
                        return Err("Unterminated escape sequence".to_string());
                    }
                }
                _ => result.push(c),
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        let mut array = Vec::new();
        match self.chars.next() {
            Some('[') => {}
            _ => return Err("Expected '['".to_string()),
        }

        self.skip_whitespace();
        if let Some(']') = self.chars.peek() {
            self.chars.next();
            return Ok(JsonValue::Array(array));
        }

        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            match self.chars.next() {
                Some(',') => continue,
                Some(']') => break,
                _ => return Err("Expected ',' or ']'".to_string()),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        let mut object = HashMap::new();
        match self.chars.next() {
            Some('{') => {}
            _ => return Err("Expected '{'".to_string()),
        }

        self.skip_whitespace();
        if let Some('}') = self.chars.peek() {
            self.chars.next();
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };

            self.skip_whitespace();
            match self.chars.next() {
                Some(':') => {}
                _ => return Err("Expected ':'".to_string()),
            }

            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();

            match self.chars.next() {
                Some(',') => continue,
                Some('}') => break,
                _ => return Err("Expected ',' or '}'".to_string()),
            }
        }

        Ok(JsonValue::Object(object))
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
                JsonValue::Number(3.0)
            ]))
        );
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new("{\"key\": \"value\"}");
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));
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
        self.pos += 1;
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
                result.push(match escaped {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'b' => '\x08',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
                });
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
        let mut has_exp = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            match c {
                '0'..='9' => self.pos += 1,
                '.' => {
                    if has_dot || has_exp {
                        return Err("Invalid number format".to_string());
                    }
                    has_dot = true;
                    self.pos += 1;
                }
                'e' | 'E' => {
                    if has_exp {
                        return Err("Invalid number format".to_string());
                    }
                    has_exp = true;
                    self.pos += 1;
                    if self.pos < self.input.len() {
                        let next = self.input.chars().nth(self.pos).unwrap();
                        if next == '+' || next == '-' {
                            self.pos += 1;
                        }
                    }
                }
                '+' | '-' => {
                    if self.pos != start {
                        return Err("Invalid number format".to_string());
                    }
                    self.pos += 1;
                }
                _ => break,
            }
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1;
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
        self.pos += 1;
        self.skip_whitespace();
        let mut object = HashMap::new();

        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == '}' {
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
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos).unwrap() != ':' {
                return Err("Expected ':' after key".to_string());
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
            return Err("Trailing characters after JSON".to_string());
        }
        Ok(result)
    }
}

pub fn pretty_print_json(value: &JsonValue, indent: usize) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| pretty_print_json(v, indent + 2))
                    .collect();
                let indent_str = " ".repeat(indent);
                let inner_indent = " ".repeat(indent + 2);
                format!("[\n{}", inner_indent)
                    + &items.join(&format!(",\n{}", inner_indent))
                    + &format!("\n{}]", indent_str)
            }
        }
        JsonValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let mut entries: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "\"{}\": {}",
                            k,
                            pretty_print_json(v, indent + 2)
                        )
                    })
                    .collect();
                entries.sort();
                let indent_str = " ".repeat(indent);
                let inner_indent = " ".repeat(indent + 2);
                format!("{{\n{}", inner_indent)
                    + &entries.join(&format!(",\n{}", inner_indent))
                    + &format!("\n{}}}", indent_str)
            }
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
        let mut parser = JsonParser::new("\"hello world\"");
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
        let mut parser = JsonParser::new("{\"key\": \"value\"}");
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));
    }

    #[test]
    fn test_pretty_print() {
        let json = JsonValue::Object({
            let mut map = HashMap::new();
            map.insert("name".to_string(), JsonValue::String("John".to_string()));
            map.insert("age".to_string(), JsonValue::Number(30.0));
            map.insert("active".to_string(), JsonValue::Bool(true));
            map
        });

        let printed = pretty_print_json(&json, 0);
        assert!(printed.contains("\"name\": \"John\""));
        assert!(printed.contains("\"age\": 30"));
        assert!(printed.contains("\"active\": true"));
    }
}