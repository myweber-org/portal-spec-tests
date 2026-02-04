
use std::collections::HashMap;

pub struct DataCleaner;

impl DataCleaner {
    pub fn clean_string_vector(data: Vec<Option<String>>) -> Vec<String> {
        data.into_iter()
            .filter_map(|item| item.map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn clean_hashmap(data: HashMap<String, Option<String>>) -> HashMap<String, String> {
        data.into_iter()
            .filter_map(|(key, value)| {
                value
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                    .map(|v| (key, v))
            })
            .collect()
    }

    pub fn remove_null_rows<T>(data: Vec<Vec<Option<T>>>) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        data.into_iter()
            .filter(|row| row.iter().all(|cell| cell.is_some()))
            .map(|row| row.into_iter().filter_map(|cell| cell).collect())
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
        input.insert("key1".to_string(), Some("  value1  ".to_string()));
        input.insert("key2".to_string(), None);
        input.insert("key3".to_string(), Some("".to_string()));

        let result = DataCleaner::clean_hashmap(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("key1").unwrap(), "value1");
    }

    #[test]
    fn test_remove_null_rows() {
        let input = vec![
            vec![Some(1), Some(2)],
            vec![None, Some(3)],
            vec![Some(4), None],
            vec![Some(5), Some(6)],
        ];
        let result = DataCleaner::remove_null_rows(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec![1, 2]);
        assert_eq!(result[1], vec![5, 6]);
    }
}use std::io::{self, BufRead};

pub struct DataCleaner {
    threshold: f64,
}

impl DataCleaner {
    pub fn new(threshold: f64) -> Self {
        DataCleaner { threshold }
    }

    pub fn filter_outliers(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        data.iter()
            .filter(|&&value| (value - mean).abs() <= self.threshold * std_dev)
            .cloned()
            .collect()
    }

    pub fn normalize(&self, data: &[f64]) -> Vec<f64> {
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;

        if range == 0.0 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&value| (value - min) / range)
            .collect()
    }
}

fn main() {
    let cleaner = DataCleaner::new(2.0);
    let stdin = io::stdin();
    
    println!("Enter numeric values separated by spaces:");
    let mut input = String::new();
    stdin.lock().read_line(&mut input).expect("Failed to read line");

    let raw_data: Vec<f64> = input
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    if raw_data.is_empty() {
        eprintln!("No valid numeric data provided");
        return;
    }

    let filtered = cleaner.filter_outliers(&raw_data);
    let normalized = cleaner.normalize(&filtered);

    println!("Original data: {:?}", raw_data);
    println!("Filtered data: {:?}", filtered);
    println!("Normalized data: {:?}", normalized);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_outliers() {
        let cleaner = DataCleaner::new(2.0);
        let data = vec![1.0, 2.0, 3.0, 100.0];
        let filtered = cleaner.filter_outliers(&data);
        assert_eq!(filtered, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_normalize() {
        let cleaner = DataCleaner::new(2.0);
        let data = vec![10.0, 20.0, 30.0];
        let normalized = cleaner.normalize(&data);
        assert_eq!(normalized, vec![0.0, 0.5, 1.0]);
    }
}