
use std::collections::HashMap;

pub struct DataAnalyzer {
    data: Vec<f64>,
}

impl DataAnalyzer {
    pub fn new(data: Vec<f64>) -> Self {
        DataAnalyzer { data }
    }

    pub fn mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn median(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            Some((sorted[mid - 1] + sorted[mid]) / 2.0)
        } else {
            Some(sorted[mid])
        }
    }

    pub fn mode(&self) -> Option<Vec<f64>> {
        if self.data.is_empty() {
            return None;
        }
        let mut frequency_map = HashMap::new();
        for &value in &self.data {
            *frequency_map.entry(value.to_bits()).or_insert(0) += 1;
        }
        let max_count = frequency_map.values().max()?;
        let modes: Vec<f64> = frequency_map
            .iter()
            .filter(|(_, &count)| count == *max_count)
            .map(|(&bits, _)| f64::from_bits(bits))
            .collect();
        Some(modes)
    }

    pub fn variance(&self) -> Option<f64> {
        let mean = self.mean()?;
        let sum_squared_diff: f64 = self.data.iter().map(|&x| (x - mean).powi(2)).sum();
        Some(sum_squared_diff / self.data.len() as f64)
    }

    pub fn std_deviation(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }

    pub fn min(&self) -> Option<f64> {
        self.data.iter().copied().reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.data.iter().copied().reduce(f64::max)
    }

    pub fn range(&self) -> Option<f64> {
        match (self.min(), self.max()) {
            (Some(min), Some(max)) => Some(max - min),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics() {
        let analyzer = DataAnalyzer::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(analyzer.mean(), Some(3.0));
        assert_eq!(analyzer.median(), Some(3.0));
        assert_eq!(analyzer.mode(), Some(vec![1.0, 2.0, 3.0, 4.0, 5.0]));
        assert_eq!(analyzer.variance(), Some(2.0));
        assert_eq!(analyzer.std_deviation(), Some(2.0_f64.sqrt()));
        assert_eq!(analyzer.min(), Some(1.0));
        assert_eq!(analyzer.max(), Some(5.0));
        assert_eq!(analyzer.range(), Some(4.0));
    }

    #[test]
    fn test_empty_data() {
        let analyzer = DataAnalyzer::new(vec![]);
        assert_eq!(analyzer.mean(), None);
        assert_eq!(analyzer.median(), None);
        assert_eq!(analyzer.mode(), None);
        assert_eq!(analyzer.variance(), None);
        assert_eq!(analyzer.std_deviation(), None);
        assert_eq!(analyzer.min(), None);
        assert_eq!(analyzer.max(), None);
        assert_eq!(analyzer.range(), None);
    }

    #[test]
    fn test_single_value() {
        let analyzer = DataAnalyzer::new(vec![42.0]);
        assert_eq!(analyzer.mean(), Some(42.0));
        assert_eq!(analyzer.median(), Some(42.0));
        assert_eq!(analyzer.mode(), Some(vec![42.0]));
        assert_eq!(analyzer.variance(), Some(0.0));
        assert_eq!(analyzer.std_deviation(), Some(0.0));
        assert_eq!(analyzer.min(), Some(42.0));
        assert_eq!(analyzer.max(), Some(42.0));
        assert_eq!(analyzer.range(), Some(0.0));
    }
}