
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;
            if record.len() >= 3 {
                let id: u32 = record[0].parse()?;
                let value: f64 = record[1].parse()?;
                let category = record[2].to_string();

                match DataRecord::new(id, value, category) {
                    Ok(data_record) => self.records.push(data_record),
                    Err(e) => eprintln!("Skipping invalid record: {}", e),
                }
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let count = self.records.len() as f64;
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        let mean = total / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());
        
        let record = DataRecord::new(2, -10.0, "test".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        let total = processor.calculate_total();
        assert!((total - 46.5).abs() < 0.001);
    }
}