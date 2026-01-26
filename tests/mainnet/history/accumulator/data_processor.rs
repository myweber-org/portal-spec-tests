
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.values.is_empty() && self.id > 0
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        DataStatistics::new(mean, variance.sqrt(), count as usize).into()
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

#[derive(Debug)]
pub struct DataStatistics {
    pub mean: f64,
    pub std_dev: f64,
    pub sample_count: usize,
}

impl DataStatistics {
    pub fn new(mean: f64, std_dev: f64, sample_count: usize) -> Self {
        Self {
            mean,
            std_dev,
            sample_count,
        }
    }

    pub fn is_normal_distribution(&self) -> bool {
        self.std_dev > 0.0 && self.sample_count >= 30
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataStatistics> {
    records
        .iter()
        .filter(|r| r.is_valid())
        .filter_map(|r| r.calculate_statistics())
        .collect()
}

pub fn filter_outliers(records: &[DataRecord], threshold: f64) -> Vec<DataRecord> {
    let stats: Vec<_> = process_records(records);
    let overall_mean: f64 = stats.iter().map(|s| s.mean).sum::<f64>() / stats.len() as f64;
    let overall_std: f64 = stats.iter().map(|s| s.std_dev).sum::<f64>() / stats.len() as f64;

    records
        .iter()
        .filter(|record| {
            if let Some(stat) = record.calculate_statistics() {
                let z_score = (stat.mean - overall_mean).abs() / overall_std;
                z_score <= threshold
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let empty_record = DataRecord::new(1, vec![]);
        assert!(!empty_record.is_valid());

        let zero_id_record = DataRecord::new(0, vec![1.0]);
        assert!(!zero_id_record.is_valid());
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        let stats = record.calculate_statistics().unwrap();
        
        assert_eq!(stats.mean, 2.0);
        assert_eq!(stats.sample_count, 3);
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, vec![1.0]);
        record.add_metadata("source".to_string(), "sensor_a".to_string());
        
        assert_eq!(record.metadata.get("source"), Some(&"sensor_a".to_string()));
    }
}