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
}