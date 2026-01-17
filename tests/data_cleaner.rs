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
}