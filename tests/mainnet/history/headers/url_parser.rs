use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MalformedQuery,
    InvalidEncoding,
}

pub fn parse_query_string(query: &str) -> Result<HashMap<String, String>, ParseError> {
    if query.is_empty() {
        return Ok(HashMap::new());
    }

    let mut params = HashMap::new();
    
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        
        let key = parts.next().ok_or(ParseError::MalformedQuery)?;
        let value = parts.next().unwrap_or("");
        
        if key.is_empty() {
            return Err(ParseError::MalformedQuery);
        }
        
        let decoded_key = percent_decode(key).map_err(|_| ParseError::InvalidEncoding)?;
        let decoded_value = percent_decode(value).map_err(|_| ParseError::InvalidEncoding)?;
        
        params.insert(decoded_key, decoded_value);
    }
    
    Ok(params)
}

fn percent_decode(input: &str) -> Result<String, ()> {
    let mut result = Vec::new();
    let mut bytes = input.bytes();
    
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let hex_high = bytes.next().ok_or(())?;
            let hex_low = bytes.next().ok_or(())?;
            
            let decoded = hex_to_byte(hex_high, hex_low).ok_or(())?;
            result.push(decoded);
        } else if byte == b'+' {
            result.push(b' ');
        } else {
            result.push(byte);
        }
    }
    
    String::from_utf8(result).map_err(|_| ())
}

fn hex_to_byte(high: u8, low: u8) -> Option<u8> {
    let to_hex = |c: u8| match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    };
    
    let high_val = to_hex(high)?;
    let low_val = to_hex(low)?;
    
    Some((high_val << 4) | low_val)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_query() {
        let result = parse_query_string("").unwrap();
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_single_param() {
        let result = parse_query_string("name=john").unwrap();
        assert_eq!(result.get("name"), Some(&"john".to_string()));
    }
    
    #[test]
    fn test_multiple_params() {
        let result = parse_query_string("name=john&age=25&city=new+york").unwrap();
        assert_eq!(result.get("name"), Some(&"john".to_string()));
        assert_eq!(result.get("age"), Some(&"25".to_string()));
        assert_eq!(result.get("city"), Some(&"new york".to_string()));
    }
    
    #[test]
    fn test_percent_encoding() {
        let result = parse_query_string("message=hello%20world%21").unwrap();
        assert_eq!(result.get("message"), Some(&"hello world!".to_string()));
    }
    
    #[test]
    fn test_malformed_query() {
        let result = parse_query_string("=value");
        assert_eq!(result, Err(ParseError::MalformedQuery));
    }
    
    #[test]
    fn test_invalid_encoding() {
        let result = parse_query_string("test=hello%2Gworld");
        assert_eq!(result, Err(ParseError::InvalidEncoding));
    }
}