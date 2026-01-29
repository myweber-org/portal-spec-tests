use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
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

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "2024-01-15T10:30:00Z");
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.timestamp, "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "2024-01-15T10:30:00Z");
        assert!(record.is_err());
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 10.0, "timestamp1").unwrap());
        processor.add_record(DataRecord::new(2, 20.0, "timestamp2").unwrap());
        processor.add_record(DataRecord::new(3, 30.0, "timestamp3").unwrap());

        assert_eq!(processor.calculate_average(), Some(20.0));
    }

    #[test]
    fn test_csv_operations() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 15.5, "2024-01-15T10:30:00Z").unwrap());
        processor.add_record(DataRecord::new(2, 25.5, "2024-01-15T11:30:00Z").unwrap());

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        assert!(processor.save_to_csv(path).is_ok());

        let mut new_processor = DataProcessor::new();
        assert!(new_processor.load_from_csv(path).is_ok());
        assert_eq!(new_processor.get_record_count(), 2);
    }
}