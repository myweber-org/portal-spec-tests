
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Vec<f64> {
        values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .map(|&v| v * 2.0)
            .collect()
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        let count = values.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(0.3).unwrap();
        let values = vec![0.1, 0.4, 0.2, 0.5, 0.6];
        let result = processor.process_values(&values);
        assert_eq!(result, vec![0.8, 1.0, 1.2]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }
}use csv::Reader;
use std::error::Error;
use std::fs::File;

pub struct DataProcessor {
    data: Vec<Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<f64> = record.iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            
            if !row.is_empty() {
                self.data.push(row);
            }
        }
        
        Ok(())
    }

    pub fn calculate_column_averages(&self) -> Vec<f64> {
        if self.data.is_empty() {
            return Vec::new();
        }
        
        let num_columns = self.data[0].len();
        let mut sums = vec![0.0; num_columns];
        let mut counts = vec![0; num_columns];
        
        for row in &self.data {
            for (i, &value) in row.iter().enumerate() {
                if i < num_columns {
                    sums[i] += value;
                    counts[i] += 1;
                }
            }
        }
        
        sums.iter()
            .zip(counts.iter())
            .map(|(&sum, &count)| if count > 0 { sum / count as f64 } else { 0.0 })
            .collect()
    }

    pub fn filter_by_threshold(&self, column_index: usize, threshold: f64) -> Vec<Vec<f64>> {
        self.data.iter()
            .filter(|row| column_index < row.len() && row[column_index] > threshold)
            .cloned()
            .collect()
    }

    pub fn get_summary_stats(&self) -> (usize, usize) {
        let rows = self.data.len();
        let columns = if rows > 0 { self.data[0].len() } else { 0 };
        (rows, columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1.0,2.0,3.0").unwrap();
        writeln!(temp_file, "4.0,5.0,6.0").unwrap();
        writeln!(temp_file, "7.0,8.0,9.0").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let (rows, cols) = processor.get_summary_stats();
        assert_eq!(rows, 3);
        assert_eq!(cols, 3);
        
        let averages = processor.calculate_column_averages();
        assert_eq!(averages, vec![4.0, 5.0, 6.0]);
        
        let filtered = processor.filter_by_threshold(1, 4.5);
        assert_eq!(filtered.len(), 2);
    }
}