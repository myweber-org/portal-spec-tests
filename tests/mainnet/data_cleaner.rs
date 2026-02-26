
use std::collections::HashMap;

pub struct DataCleaner;

impl DataCleaner {
    pub fn clean_string_vector(data: Vec<Option<String>>) -> Vec<String> {
        data.into_iter()
            .filter_map(|item| item)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn clean_hashmap(data: HashMap<String, Option<f64>>) -> HashMap<String, f64> {
        data.into_iter()
            .filter_map(|(key, value)| value.map(|v| (key, v)))
            .filter(|(_, value)| value.is_finite())
            .collect()
    }

    pub fn remove_outliers(data: &[f64], threshold: f64) -> Vec<f64> {
        if data.is_empty() {
            return Vec::new();
        }

        let mean: f64 = data.iter().sum::<f64>() / data.len() as f64;
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        data.iter()
            .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string_vector() {
        let input = vec![
            Some("  hello  ".to_string()),
            None,
            Some("".to_string()),
            Some("world".to_string()),
        ];
        let result = DataCleaner::clean_string_vector(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_clean_hashmap() {
        let mut input = HashMap::new();
        input.insert("a".to_string(), Some(1.5));
        input.insert("b".to_string(), None);
        input.insert("c".to_string(), Some(f64::NAN));

        let result = DataCleaner::clean_hashmap(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("a"), Some(&1.5));
    }

    #[test]
    fn test_remove_outliers() {
        let data = vec![1.0, 2.0, 3.0, 100.0];
        let result = DataCleaner::remove_outliers(&data, 2.0);
        assert_eq!(result, vec![1.0, 2.0, 3.0]);
    }
}