use std::str::FromStr;

pub fn filter_numbers<T: FromStr>(items: Vec<String>) -> Vec<T> {
    items
        .into_iter()
        .filter_map(|s| s.parse::<T>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_numbers() {
        let input = vec![
            "42".to_string(),
            "hello".to_string(),
            "3.14".to_string(),
            "world".to_string(),
            "100".to_string(),
        ];
        let result: Vec<i32> = filter_numbers(input);
        assert_eq!(result, vec![42, 100]);
    }
}use regex::Regex;

pub fn extract_numbers(input: &str) -> String {
    let re = Regex::new(r"[^0-9]").unwrap();
    re.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numbers() {
        assert_eq!(extract_numbers("abc123def456"), "123456");
        assert_eq!(extract_numbers("phone: 555-1234"), "5551234");
        assert_eq!(extract_numbers("no digits here"), "");
        assert_eq!(extract_numbers(""), "");
    }
}