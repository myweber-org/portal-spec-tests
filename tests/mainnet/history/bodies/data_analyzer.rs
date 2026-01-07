use std::collections::HashMap;

pub struct DataAnalyzer;

impl DataAnalyzer {
    pub fn new() -> Self {
        DataAnalyzer
    }

    pub fn calculate_mean(&self, data: &[f64]) -> Option<f64> {
        if data.is_empty() {
            return None;
        }
        let sum: f64 = data.iter().sum();
        Some(sum / data.len() as f64)
    }

    pub fn calculate_median(&self, mut data: Vec<f64>) -> Option<f64> {
        if data.is_empty() {
            return None;
        }
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = data.len() / 2;
        if data.len() % 2 == 0 {
            Some((data[mid - 1] + data[mid]) / 2.0)
        } else {
            Some(data[mid])
        }
    }

    pub fn calculate_mode(&self, data: &[f64]) -> Option<f64> {
        if data.is_empty() {
            return None;
        }
        let mut frequency_map = HashMap::new();
        for &value in data {
            *frequency_map.entry(value.to_bits()).or_insert(0) += 1;
        }
        frequency_map
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(bits, _)| f64::from_bits(bits))
    }

    pub fn calculate_standard_deviation(&self, data: &[f64]) -> Option<f64> {
        if data.len() < 2 {
            return None;
        }
        let mean = self.calculate_mean(data)?;
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (data.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn calculate_range(&self, data: &[f64]) -> Option<f64> {
        if data.is_empty() {
            return None;
        }
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        Some(max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_calculation() {
        let analyzer = DataAnalyzer::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(analyzer.calculate_mean(&data), Some(3.0));
    }

    #[test]
    fn test_median_calculation() {
        let analyzer = DataAnalyzer::new();
        let data = vec![5.0, 2.0, 1.0, 4.0, 3.0];
        assert_eq!(analyzer.calculate_median(data), Some(3.0));
    }

    #[test]
    fn test_mode_calculation() {
        let analyzer = DataAnalyzer::new();
        let data = vec![1.0, 2.0, 2.0, 3.0, 4.0];
        assert_eq!(analyzer.calculate_mode(&data), Some(2.0));
    }

    #[test]
    fn test_standard_deviation() {
        let analyzer = DataAnalyzer::new();
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let result = analyzer.calculate_standard_deviation(&data).unwrap();
        assert!((result - 2.138).abs() < 0.001);
    }

    #[test]
    fn test_range_calculation() {
        let analyzer = DataAnalyzer::new();
        let data = vec![10.0, 5.0, 20.0, 15.0, 25.0];
        assert_eq!(analyzer.calculate_range(&data), Some(20.0));
    }
}