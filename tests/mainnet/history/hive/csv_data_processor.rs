
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_active_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn save_filtered_to_csv(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in filtered {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn find_min_value(&self) -> Option<&DataRecord> {
        self.records
            .iter()
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_data = vec![
            DataRecord { id: 1, category: "A".to_string(), value: 10.5, active: true },
            DataRecord { id: 2, category: "B".to_string(), value: 20.3, active: false },
            DataRecord { id: 3, category: "A".to_string(), value: 15.7, active: true },
        ];

        processor.records = test_data;

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.001);

        let active_records = processor.get_active_records();
        assert_eq!(active_records.len(), 2);

        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 2);

        let min_record = processor.find_min_value().unwrap();
        assert_eq!(min_record.id, 1);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let temp_file = NamedTempFile::new()?;
        let test_path = temp_file.path().to_str().unwrap();

        let mut processor = DataProcessor::new();
        
        let test_data = vec![
            DataRecord { id: 1, category: "Test".to_string(), value: 42.0, active: true },
            DataRecord { id: 2, category: "Test".to_string(), value: 84.0, active: false },
        ];

        processor.records = test_data;

        processor.save_filtered_to_csv(test_path, "Test")?;

        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(test_path)?;

        assert_eq!(new_processor.records.len(), 2);
        
        Ok(())
    }
}