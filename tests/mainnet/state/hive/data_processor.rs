
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let value: f64 = line.trim().parse()?;
            self.data.push(value);
        }

        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }

        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;

        Some(variance.sqrt())
    }

    pub fn get_min_max(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }

        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some((min, max))
    }

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        if self.data.len() < 2 {
            return self.data.clone();
        }

        let mean = match self.calculate_mean() {
            Some(m) => m,
            None => return self.data.clone(),
        };

        let std_dev = match self.calculate_standard_deviation() {
            Some(s) => s,
            None => return self.data.clone(),
        };

        self.data
            .iter()
            .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
            .cloned()
            .collect()
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        assert!(processor.load_from_csv(temp_file.path()).is_ok());
        assert_eq!(processor.data_count(), 5);
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 18.1).abs() < 0.01);
        
        let (min, max) = processor.get_min_max().unwrap();
        assert_eq!(min, 10.5);
        assert_eq!(max, 25.1);
        
        let filtered = processor.filter_outliers(2.0);
        assert_eq!(filtered.len(), 5);
    }
}