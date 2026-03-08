
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut values = Vec::new();

        for result in rdr.records() {
            let record = result?;
            if let Some(field) = record.get(0) {
                if let Ok(value) = field.parse::<f64>() {
                    values.push(value);
                }
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.calculate_mean()?;
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.values.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            count: self.values.len(),
            mean: self.calculate_mean(),
            std_dev: self.calculate_standard_deviation(),
        }
    }
}

pub struct DataSummary {
    pub count: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
}

impl std::fmt::Display for DataSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Count: {}", self.count)?;
        if let Some(mean) = self.mean {
            write!(f, ", Mean: {:.4}", mean)?;
        }
        if let Some(std_dev) = self.std_dev {
            write!(f, ", Std Dev: {:.4}", std_dev)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let dataset = DataSet::new();
        assert_eq!(dataset.calculate_mean(), None);
        assert_eq!(dataset.calculate_standard_deviation(), None);
    }

    #[test]
    fn test_basic_statistics() {
        let mut dataset = DataSet::new();
        dataset.add_value(10.0);
        dataset.add_value(20.0);
        dataset.add_value(30.0);
        
        assert_eq!(dataset.calculate_mean(), Some(20.0));
        assert!(dataset.calculate_standard_deviation().unwrap() > 0.0);
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "value")?;
        writeln!(temp_file, "5.5")?;
        writeln!(temp_file, "10.2")?;
        writeln!(temp_file, "15.8")?;
        
        let dataset = DataSet::from_csv(temp_file.path())?;
        assert_eq!(dataset.values.len(), 3);
        assert!(dataset.calculate_mean().unwrap() > 0.0);
        
        Ok(())
    }
}