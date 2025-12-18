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
}