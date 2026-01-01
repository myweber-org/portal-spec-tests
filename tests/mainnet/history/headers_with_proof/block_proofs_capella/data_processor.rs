use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: &str) -> Result<Self, String> {
        if value < 0.0 || value > 1000.0 {
            return Err(format!("Value {} out of valid range [0, 1000]", value));
        }
        if timestamp.is_empty() {
            return Err("Timestamp cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            timestamp: timestamp.to_string(),
        })
    }

    pub fn calculate_normalized(&self) -> f64 {
        self.value / 1000.0
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::Reader::from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = csv::Writer::from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 500.0, "2024-01-15T10:30:00Z");
        assert!(record.is_ok());
        
        let invalid_record = DataRecord::new(2, -10.0, "2024-01-15T10:30:00Z");
        assert!(invalid_record.is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, 100.0, "2024-01-15T10:30:00Z").unwrap();
        let record2 = DataRecord::new(2, 200.0, "2024-01-15T11:30:00Z").unwrap();
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        assert_eq!(processor.records.len(), 2);
        assert_eq!(processor.calculate_average(), Some(150.0));
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, 100.0, "2024-01-15T10:30:00Z").unwrap();
        let record2 = DataRecord::new(2, 200.0, "2024-01-15T11:30:00Z").unwrap();
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path)?;
        
        assert_eq!(new_processor.records.len(), 2);
        Ok(())
    }
}