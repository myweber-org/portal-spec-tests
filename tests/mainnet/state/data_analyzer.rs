use std::collections::HashMap;

pub struct DataAnalyzer {
    data: Vec<f64>,
}

impl DataAnalyzer {
    pub fn new() -> Self {
        DataAnalyzer { data: Vec::new() }
    }

    pub fn add_data_point(&mut self, value: f64) {
        self.data.push(value);
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_std_dev(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        let mean = self.calculate_mean().unwrap();
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn frequency_distribution(&self, bins: usize) -> Option<HashMap<usize, usize>> {
        if self.data.is_empty() || bins == 0 {
            return None;
        }
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;
        let bin_width = range / bins as f64;

        let mut distribution = HashMap::new();
        for &value in &self.data {
            let bin_index = ((value - min) / bin_width).floor() as usize;
            let bin_index = bin_index.min(bins - 1);
            *distribution.entry(bin_index).or_insert(0) += 1;
        }
        Some(distribution)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}