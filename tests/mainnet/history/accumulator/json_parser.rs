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

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Empty input".to_string());
    }
    parse_value(trimmed).and_then(|(val, remaining)| {
        if remaining.trim().is_empty() {
            Ok(val)
        } else {
            Err("Unexpected trailing characters".to_string())
        }
    })
}

fn parse_value(input: &str) -> Result<(JsonValue, &str), String> {
    let input = input.trim();
    if input.starts_with('{') {
        parse_object(input)
    } else if input.starts_with('[') {
        parse_array(input)
    } else if input.starts_with('"') {
        parse_string(input)
    } else if input.starts_with("null") {
        Ok((JsonValue::Null, &input[4..]))
    } else if input.starts_with("true") {
        Ok((JsonValue::Bool(true), &input[4..]))
    } else if input.starts_with("false") {
        Ok((JsonValue::Bool(false), &input[5..]))
    } else if let Some(num) = parse_number(input) {
        num
    } else {
        Err(format!("Invalid JSON value: {}", &input[0..input.len().min(10)]))
    }
}

fn parse_object(input: &str) -> Result<(JsonValue, &str), String> {
    let mut map = HashMap::new();
    let mut chars = input.chars();
    
    if chars.next() != Some('{') {
        return Err("Expected '{{'".to_string());
    }
    
    let mut remaining = &input[1..];
    remaining = remaining.trim();
    
    if remaining.starts_with('}') {
        return Ok((JsonValue::Object(map), &remaining[1..]));
    }
    
    loop {
        remaining = remaining.trim();
        if !remaining.starts_with('"') {
            return Err("Expected string key".to_string());
        }
        
        let (key, rest) = parse_string(remaining)?;
        let key_str = match key {
            JsonValue::String(s) => s,
            _ => unreachable!(),
        };
        
        remaining = rest.trim();
        if !remaining.starts_with(':') {
            return Err("Expected ':'".to_string());
        }
        remaining = &remaining[1..];
        
        let (value, rest) = parse_value(remaining)?;
        map.insert(key_str, value);
        remaining = rest.trim();
        
        if remaining.starts_with('}') {
            return Ok((JsonValue::Object(map), &remaining[1..]));
        }
        
        if !remaining.starts_with(',') {
            return Err("Expected ',' or '}}'".to_string());
        }
        remaining = &remaining[1..];
    }
}

fn parse_array(input: &str) -> Result<(JsonValue, &str), String> {
    let mut vec = Vec::new();
    let mut chars = input.chars();
    
    if chars.next() != Some('[') {
        return Err("Expected '['".to_string());
    }
    
    let mut remaining = &input[1..];
    remaining = remaining.trim();
    
    if remaining.starts_with(']') {
        return Ok((JsonValue::Array(vec), &remaining[1..]));
    }
    
    loop {
        let (value, rest) = parse_value(remaining)?;
        vec.push(value);
        remaining = rest.trim();
        
        if remaining.starts_with(']') {
            return Ok((JsonValue::Array(vec), &remaining[1..]));
        }
        
        if !remaining.starts_with(',') {
            return Err("Expected ',' or ']'".to_string());
        }
        remaining = &remaining[1..];
    }
}

fn parse_string(input: &str) -> Result<(JsonValue, &str), String> {
    let mut chars = input.chars();
    if chars.next() != Some('"') {
        return Err("Expected '\"'".to_string());
    }
    
    let mut result = String::new();
    let mut escape = false;
    
    for (i, c) in input[1..].chars().enumerate() {
        if escape {
            match c {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'b' => result.push('\x08'),
                'f' => result.push('\x0c'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                _ => return Err(format!("Invalid escape sequence: \\{}", c)),
            }
            escape = false;
        } else if c == '\\' {
            escape = true;
        } else if c == '"' {
            let remaining = &input[i + 2..];
            return Ok((JsonValue::String(result), remaining));
        } else {
            result.push(c);
        }
    }
    
    Err("Unterminated string".to_string())
}

fn parse_number(input: &str) -> Option<Result<(JsonValue, &str), String>> {
    let mut end = 0;
    let chars: Vec<char> = input.chars().collect();
    
    if chars.get(end) == Some(&'-') {
        end += 1;
    }
    
    if chars.get(end) == Some(&'0') {
        end += 1;
    } else if chars.get(end).map(|c| c.is_ascii_digit()).unwrap_or(false) {
        end += 1;
        while chars.get(end).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            end += 1;
        }
    } else {
        return None;
    }
    
    if chars.get(end) == Some(&'.') {
        end += 1;
        if !chars.get(end).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return Some(Err("Invalid number format".to_string()));
        }
        end += 1;
        while chars.get(end).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            end += 1;
        }
    }
    
    if chars.get(end).map(|c| *c == 'e' || *c == 'E').unwrap_or(false) {
        end += 1;
        if chars.get(end).map(|c| *c == '+' || *c == '-').unwrap_or(false) {
            end += 1;
        }
        if !chars.get(end).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return Some(Err("Invalid exponent format".to_string()));
        }
        end += 1;
        while chars.get(end).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            end += 1;
        }
    }
    
    let num_str = &input[..end];
    match num_str.parse::<f64>() {
        Ok(num) => Some(Ok((JsonValue::Number(num), &input[end..]))),
        Err(_) => Some(Err("Invalid number".to_string())),
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
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.pos += 1;
                return Ok(());
            }
        }
        Err(format!("Expected '{}' at position {}", expected, self.pos))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.pos += 1;
                return Ok(result);
            } else if ch == '\\' {
                self.pos += 1;
                let escaped = self.peek().ok_or("Unexpected end of input after escape")?;
                match escaped {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\u{0008}'),
                    'f' => result.push('\u{000C}'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
                }
                self.pos += 1;
            } else {
                result.push(ch);
                self.pos += 1;
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        self.skip_whitespace();
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == 'e' || ch == 'E' || ch == '+' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        num_str.parse().map_err(|e| format!("Invalid number: {}", e))
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if let Some(ch) = self.peek() {
            if ch == ']' {
                self.pos += 1;
                return Ok(array);
            }
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == ']' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    continue;
                } else {
                    return Err(format!("Expected ',' or ']' at position {}", self.pos));
                }
            } else {
                return Err("Unexpected end of input in array".to_string());
            }
        }
        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if let Some(ch) = self.peek() {
            if ch == '}' {
                self.pos += 1;
                return Ok(map);
            }
        }
        loop {
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == '}' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    continue;
                } else {
                    return Err(format!("Expected ',' or '}}' at position {}", self.pos));
                }
            } else {
                return Err("Unexpected end of input in object".to_string());
            }
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('"') => {
                let s = self.parse_string()?;
                Ok(JsonValue::String(s))
            }
            Some('{') => {
                let obj = self.parse_object()?;
                Ok(JsonValue::Object(obj))
            }
            Some('[') => {
                let arr = self.parse_array()?;
                Ok(JsonValue::Array(arr))
            }
            Some('t') => {
                if self.input[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
                    self.pos += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("Expected 'true'".to_string())
                }
            }
            Some('f') => {
                if self.input[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
                    self.pos += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("Expected 'false'".to_string())
                }
            }
            Some('n') => {
                if self.input[self.pos..].starts_with(&['n', 'u', 'l', 'l']) {
                    self.pos += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err("Expected 'null'".to_string())
                }
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                let num = self.parse_number()?;
                Ok(JsonValue::Number(num))
            }
            _ => Err("Unexpected character".to_string()),
        }
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
    fn test_parse_simple_json() {
        let mut parser = JsonParser::new(r#"{"name": "test", "value": 42.5}"#);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new(r#"[1, 2, "three", true]"#);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}