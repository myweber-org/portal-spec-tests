use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new(data: Vec<Vec<Option<String>>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self) {
        self.data.retain(|row| {
            row.iter().all(|cell| cell.is_some())
        });
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| {
            let row_string: String = row
                .iter()
                .map(|cell| cell.as_ref().unwrap_or(&"NULL".to_string()))
                .collect::<Vec<&String>>()
                .join("|");
            seen.insert(row_string)
        });
    }

    pub fn get_data(&self) -> &Vec<Vec<Option<String>>> {
        &self.data
    }

    pub fn count_rows(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_operations() {
        let mut cleaner = DataCleaner::new(vec![
            vec![Some("A".to_string()), Some("1".to_string())],
            vec![Some("B".to_string()), None],
            vec![Some("A".to_string()), Some("1".to_string())],
        ]);

        assert_eq!(cleaner.count_rows(), 3);
        
        cleaner.remove_null_rows();
        assert_eq!(cleaner.count_rows(), 2);
        
        cleaner.deduplicate();
        assert_eq!(cleaner.count_rows(), 1);
    }
}use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    threshold: f64,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>, threshold: f64) -> Self {
        DataCleaner { data, threshold }
    }

    pub fn remove_outliers(&mut self) -> Vec<f64> {
        if self.data.len() < 4 {
            return self.data.clone();
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = Self::calculate_quantile(&sorted_data, 0.25);
        let q3 = Self::calculate_quantile(&sorted_data, 0.75);
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        self.data
            .iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let mean = sum / self.data.len() as f64;

        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.data.len() as f64;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), self.data.len() as f64);

        stats
    }

    fn calculate_quantile(sorted_data: &[f64], percentile: f64) -> f64 {
        let n = sorted_data.len();
        let index = percentile * (n - 1) as f64;
        
        let lower_idx = index.floor() as usize;
        let upper_idx = index.ceil() as usize;
        
        if lower_idx == upper_idx {
            sorted_data[lower_idx]
        } else {
            let weight = index - lower_idx as f64;
            sorted_data[lower_idx] * (1.0 - weight) + sorted_data[upper_idx] * weight
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data, 1.5);
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data, 1.5);
        let stats = cleaner.get_statistics();
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
    }
}