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
                let parsed = self.input[start..self.pos].to_string();
                self.pos += 1;
                return Ok(JsonValue::String(parsed));
            } else if c == '\\' {
                // Simple escape handling
                self.pos += 1;
                if self.pos >= self.input.len() {
                    return Err("Unterminated escape sequence".to_string());
                }
                let next = self.input.chars().nth(self.pos).unwrap();
                result.push(match next {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'b' => '\x08',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => return Err(format!("Invalid escape sequence: \\{}", next)),
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
        self.pos += 1; // Skip '['
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == ']' {
            self.pos += 1;
            return Ok(JsonValue::Array(array));
        }

        loop {
            self.skip_whitespace();
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
                continue;
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
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
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
                continue;
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

        let mut parser = JsonParser::new("1.23e4");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(12300.0)));
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
        let mut parser = JsonParser::new(r#"{"key": "value", "num": 42}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        expected.insert("num".to_string(), JsonValue::Number(42.0));
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
        self.pos += 1; // Skip opening quote
        let start = self.pos;
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '"' {
                break;
            }
            result.push(c);
            self.pos += 1;
        }

        if self.pos >= self.input.len() {
            return Err("Unterminated string".to_string());
        }

        self.pos += 1; // Skip closing quote
        Ok(JsonValue::String(result))
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
            } else if !c.is_digit(10) && !(c == '-' && self.pos == start) {
                break;
            }
            self.pos += 1;
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '['
        let mut array = Vec::new();

        self.skip_whitespace();
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
        let mut object = HashMap::new();

        self.skip_whitespace();
        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            if self.input.chars().nth(self.pos).unwrap() != '"' {
                return Err("Expected string key".to_string());
            }

            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos).unwrap() != ':' {
                return Err("Expected ':'".to_string());
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
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse(), Ok(expected));
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

#[derive(Debug)]
pub struct ParseError {
    message: String,
    position: usize,
}

pub fn parse_json(input: &str) -> Result<JsonValue, ParseError> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    parse_value(&chars, &mut index)
}

fn parse_value(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    skip_whitespace(chars, index);
    
    if *index >= chars.len() {
        return Err(ParseError {
            message: "Unexpected end of input".to_string(),
            position: *index,
        });
    }
    
    match chars[*index] {
        'n' => parse_literal(chars, index, "null", JsonValue::Null),
        't' => parse_literal(chars, index, "true", JsonValue::Bool(true)),
        'f' => parse_literal(chars, index, "false", JsonValue::Bool(false)),
        '"' => parse_string(chars, index),
        '[' => parse_array(chars, index),
        '{' => parse_object(chars, index),
        '-' | '0'..='9' => parse_number(chars, index),
        _ => Err(ParseError {
            message: format!("Unexpected character: {}", chars[*index]),
            position: *index,
        }),
    }
}

fn parse_literal(
    chars: &[char],
    index: &mut usize,
    literal: &str,
    value: JsonValue,
) -> Result<JsonValue, ParseError> {
    for expected_char in literal.chars() {
        if *index >= chars.len() || chars[*index] != expected_char {
            return Err(ParseError {
                message: format!("Expected '{}'", literal),
                position: *index,
            });
        }
        *index += 1;
    }
    Ok(value)
}

fn parse_string(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    *index += 1; // Skip opening quote
    let mut result = String::new();
    
    while *index < chars.len() && chars[*index] != '"' {
        if chars[*index] == '\\' {
            *index += 1;
            if *index >= chars.len() {
                return Err(ParseError {
                    message: "Unterminated escape sequence".to_string(),
                    position: *index,
                });
            }
            match chars[*index] {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'b' => result.push('\x08'),
                'f' => result.push('\x0c'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                _ => return Err(ParseError {
                    message: format!("Invalid escape character: {}", chars[*index]),
                    position: *index,
                }),
            }
        } else {
            result.push(chars[*index]);
        }
        *index += 1;
    }
    
    if *index >= chars.len() || chars[*index] != '"' {
        return Err(ParseError {
            message: "Unterminated string".to_string(),
            position: *index,
        });
    }
    
    *index += 1; // Skip closing quote
    Ok(JsonValue::String(result))
}

fn parse_number(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    let start = *index;
    let mut has_dot = false;
    let mut has_exponent = false;
    
    if chars[*index] == '-' {
        *index += 1;
    }
    
    while *index < chars.len() && chars[*index].is_ascii_digit() {
        *index += 1;
    }
    
    if *index < chars.len() && chars[*index] == '.' {
        has_dot = true;
        *index += 1;
        while *index < chars.len() && chars[*index].is_ascii_digit() {
            *index += 1;
        }
    }
    
    if *index < chars.len() && (chars[*index] == 'e' || chars[*index] == 'E') {
        has_exponent = true;
        *index += 1;
        if *index < chars.len() && (chars[*index] == '+' || chars[*index] == '-') {
            *index += 1;
        }
        while *index < chars.len() && chars[*index].is_ascii_digit() {
            *index += 1;
        }
    }
    
    let num_str: String = chars[start..*index].iter().collect();
    match num_str.parse::<f64>() {
        Ok(num) => Ok(JsonValue::Number(num)),
        Err(_) => Err(ParseError {
            message: format!("Invalid number: {}", num_str),
            position: start,
        }),
    }
}

fn parse_array(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    *index += 1; // Skip '['
    skip_whitespace(chars, index);
    
    let mut array = Vec::new();
    
    if *index < chars.len() && chars[*index] == ']' {
        *index += 1;
        return Ok(JsonValue::Array(array));
    }
    
    loop {
        let value = parse_value(chars, index)?;
        array.push(value);
        
        skip_whitespace(chars, index);
        if *index >= chars.len() {
            return Err(ParseError {
                message: "Unterminated array".to_string(),
                position: *index,
            });
        }
        
        if chars[*index] == ']' {
            *index += 1;
            break;
        } else if chars[*index] == ',' {
            *index += 1;
            skip_whitespace(chars, index);
        } else {
            return Err(ParseError {
                message: format!("Expected ',' or ']', found '{}'", chars[*index]),
                position: *index,
            });
        }
    }
    
    Ok(JsonValue::Array(array))
}

fn parse_object(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    *index += 1; // Skip '{'
    skip_whitespace(chars, index);
    
    let mut object = HashMap::new();
    
    if *index < chars.len() && chars[*index] == '}' {
        *index += 1;
        return Ok(JsonValue::Object(object));
    }
    
    loop {
        skip_whitespace(chars, index);
        if *index >= chars.len() || chars[*index] != '"' {
            return Err(ParseError {
                message: "Expected string key".to_string(),
                position: *index,
            });
        }
        
        let key = match parse_string(chars, index)? {
            JsonValue::String(s) => s,
            _ => unreachable!(),
        };
        
        skip_whitespace(chars, index);
        if *index >= chars.len() || chars[*index] != ':' {
            return Err(ParseError {
                message: "Expected ':'".to_string(),
                position: *index,
            });
        }
        *index += 1;
        
        let value = parse_value(chars, index)?;
        object.insert(key, value);
        
        skip_whitespace(chars, index);
        if *index >= chars.len() {
            return Err(ParseError {
                message: "Unterminated object".to_string(),
                position: *index,
            });
        }
        
        if chars[*index] == '}' {
            *index += 1;
            break;
        } else if chars[*index] == ',' {
            *index += 1;
            skip_whitespace(chars, index);
        } else {
            return Err(ParseError {
                message: format!("Expected ',' or '}}', found '{}'", chars[*index]),
                position: *index,
            });
        }
    }
    
    Ok(JsonValue::Object(object))
}

fn skip_whitespace(chars: &[char], index: &mut usize) {
    while *index < chars.len() && chars[*index].is_whitespace() {
        *index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_null() {
        assert_eq!(parse_json("null").unwrap(), JsonValue::Null);
    }
    
    #[test]
    fn test_parse_boolean() {
        assert_eq!(parse_json("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse_json("false").unwrap(), JsonValue::Bool(false));
    }
    
    #[test]
    fn test_parse_number() {
        assert_eq!(parse_json("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse_json("-3.14").unwrap(), JsonValue::Number(-3.14));
        assert_eq!(parse_json("1.23e-4").unwrap(), JsonValue::Number(1.23e-4));
    }
    
    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_json(r#""hello""#).unwrap(),
            JsonValue::String("hello".to_string())
        );
        assert_eq!(
            parse_json(r#""escape\nsequence""#).unwrap(),
            JsonValue::String("escape\nsequence".to_string())
        );
    }
    
    #[test]
    fn test_parse_array() {
        let result = parse_json("[1, 2, 3]").unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::Number(2.0));
            assert_eq!(arr[2], JsonValue::Number(3.0));
        } else {
            panic!("Expected array");
        }
    }
    
    #[test]
    fn test_parse_object() {
        let result = parse_json(r#"{"key": "value", "num": 42}"#).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 2);
            assert_eq!(obj.get("key"), Some(&JsonValue::String("value".to_string())));
            assert_eq!(obj.get("num"), Some(&JsonValue::Number(42.0)));
        } else {
            panic!("Expected object");
        }
    }
}