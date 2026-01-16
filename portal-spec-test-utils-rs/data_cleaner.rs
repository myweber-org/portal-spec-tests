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
}