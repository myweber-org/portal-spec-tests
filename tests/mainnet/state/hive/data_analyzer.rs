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

    pub fn analyze_dataset(&self, data: Vec<f64>) -> AnalysisResult {
        let mean = self.calculate_mean(&data);
        let median = self.calculate_median(data.clone());
        let mode = self.calculate_mode(&data);
        let std_dev = self.calculate_standard_deviation(&data);

        AnalysisResult {
            mean,
            median,
            mode,
            standard_deviation: std_dev,
            sample_size: data.len(),
        }
    }
}

pub struct AnalysisResult {
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub mode: Option<f64>,
    pub standard_deviation: Option<f64>,
    pub sample_size: usize,
}

impl AnalysisResult {
    pub fn display(&self) -> String {
        format!(
            "Analysis Results:\n\
            Sample Size: {}\n\
            Mean: {:.4}\n\
            Median: {:.4}\n\
            Mode: {:.4}\n\
            Standard Deviation: {:.4}",
            self.sample_size,
            self.mean.unwrap_or(f64::NAN),
            self.median.unwrap_or(f64::NAN),
            self.mode.unwrap_or(f64::NAN),
            self.standard_deviation.unwrap_or(f64::NAN)
        )
    }
}