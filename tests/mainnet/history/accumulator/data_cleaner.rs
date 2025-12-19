use std::collections::HashMap;

pub struct DataCleaner {
    threshold: f64,
}

impl DataCleaner {
    pub fn new(threshold: f64) -> Self {
        DataCleaner { threshold }
    }

    pub fn remove_outliers_iqr(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < 4 {
            return data.to_vec();
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = Self::calculate_percentile(&sorted_data, 25.0);
        let q3 = Self::calculate_percentile(&sorted_data, 75.0);
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        data.iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn analyze_dataset(&self, data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if data.is_empty() {
            return stats;
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = Self::calculate_percentile(&sorted, 50.0);

        stats.insert("mean".to_string(), mean);
        stats.insert("median".to_string(), median);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), *sorted.first().unwrap());
        stats.insert("max".to_string(), *sorted.last().unwrap());

        stats
    }

    fn calculate_percentile(sorted_data: &[f64], percentile: f64) -> f64 {
        let index = (percentile / 100.0) * (sorted_data.len() - 1) as f64;
        let lower_index = index.floor() as usize;
        let upper_index = index.ceil() as usize;
        
        if lower_index == upper_index {
            sorted_data[lower_index]
        } else {
            let weight = index - lower_index as f64;
            sorted_data[lower_index] * (1.0 - weight) + sorted_data[upper_index] * weight
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let cleaned = cleaner.remove_outliers_iqr(&data);
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = cleaner.analyze_dataset(&data);
        
        assert_eq!(stats.get("mean").unwrap().round(), 3.0);
        assert_eq!(stats.get("median").unwrap().round(), 3.0);
    }
}