use std::collections::HashSet;
use std::hash::Hash;

pub fn remove_duplicates<T: Eq + Hash + Clone>(items: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|item| seen.insert(item.clone()))
        .cloned()
        .collect()
}

pub fn normalize_strings(strings: &[String]) -> Vec<String> {
    strings
        .iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn clean_numeric_data(numbers: &[f64]) -> Vec<f64> {
    numbers
        .iter()
        .filter(|&&n| n.is_finite() && n >= 0.0)
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let data = vec![1, 2, 2, 3, 4, 4, 5];
        let result = remove_duplicates(&data);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let strings = vec![
            "  Hello  ".to_string(),
            "WORLD".to_string(),
            "  Test  ".to_string(),
        ];
        let result = normalize_strings(&strings);
        assert_eq!(result, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_clean_numeric_data() {
        let numbers = vec![1.0, f64::NAN, -5.0, 10.0, f64::INFINITY, 0.0];
        let result = clean_numeric_data(&numbers);
        assert_eq!(result, vec![1.0, 10.0, 0.0]);
    }
}