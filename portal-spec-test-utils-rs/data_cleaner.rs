use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    thresholds: HashMap<String, f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner {
            data,
            thresholds: HashMap::new(),
        }
    }

    pub fn calculate_iqr(&mut self) -> (f64, f64, f64, f64) {
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25) as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75) as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.thresholds.insert("lower_bound".to_string(), lower_bound);
        self.thresholds.insert("upper_bound".to_string(), upper_bound);
        self.thresholds.insert("iqr".to_string(), iqr);

        (q1, q3, lower_bound, upper_bound)
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        let lower = self.thresholds.get("lower_bound").unwrap_or(&f64::MIN);
        let upper = self.thresholds.get("upper_bound").unwrap_or(&f64::MAX);

        self.data
            .iter()
            .filter(|&&x| x >= *lower && x <= *upper)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("min".to_string(), self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        stats.insert("max".to_string(), self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
        stats.insert("count".to_string(), self.data.len() as f64);

        let sum: f64 = self.data.iter().sum();
        let mean = sum / self.data.len() as f64;
        stats.insert("mean".to_string(), mean);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        cleaner.calculate_iqr();
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }
}use std::collections::HashSet;

pub fn clean_and_sort_data<T: Ord + Clone>(data: &[T]) -> Vec<T> {
    let unique_set: HashSet<_> = data.iter().cloned().collect();
    let mut unique_vec: Vec<T> = unique_set.into_iter().collect();
    unique_vec.sort();
    unique_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort() {
        let input = vec![5, 2, 8, 2, 5, 1, 9];
        let result = clean_and_sort_data(&input);
        assert_eq!(result, vec![1, 2, 5, 8, 9]);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<i32> = vec![];
        let result = clean_and_sort_data(&input);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_string_data() {
        let input = vec!["banana", "apple", "cherry", "apple"];
        let result = clean_and_sort_data(&input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, items: Vec<T>) -> Vec<T> {
        let mut result = Vec::new();
        for item in items {
            if self.seen.insert(item.clone()) {
                result.push(item);
            }
        }
        result
    }

    pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
        strings
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .collect()
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }

    pub fn get_unique_count(&self) -> usize {
        self.seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_integers() {
        let mut cleaner = DataCleaner::new();
        let data = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let deduped = cleaner.deduplicate(data);
        assert_eq!(deduped, vec![1, 2, 3, 4, 5]);
        assert_eq!(cleaner.get_unique_count(), 5);
    }

    #[test]
    fn test_deduplicate_strings() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["apple", "banana", "apple", "orange"]
            .into_iter()
            .map(String::from)
            .collect();
        let deduped = cleaner.deduplicate(data);
        assert_eq!(deduped.len(), 3);
    }

    #[test]
    fn test_normalize_strings() {
        let strings = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "  RuSt  ".to_string(),
        ];
        let normalized = DataCleaner::normalize_strings(strings);
        assert_eq!(normalized, vec!["hello", "world", "rust"]);
    }

    #[test]
    fn test_reset() {
        let mut cleaner = DataCleaner::new();
        let data = vec![1, 2, 3];
        cleaner.deduplicate(data);
        assert_eq!(cleaner.get_unique_count(), 3);
        cleaner.reset();
        assert_eq!(cleaner.get_unique_count(), 0);
    }
}