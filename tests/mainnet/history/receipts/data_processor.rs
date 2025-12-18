use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    pub records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        let mut count = 0;
        for result in reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        for record in &self.records {
            writer.serialize(record)?;
        }

        writer.flush()?;
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

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= 0.0 && !record.name.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.records.len(), 0);
        assert_eq!(processor.calculate_average(), None);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            name: "Item1".to_string(),
            value: 10.5,
            category: "A".to_string(),
        });
        processor.records.push(DataRecord {
            id: 2,
            name: "Item2".to_string(),
            value: 20.0,
            category: "B".to_string(),
        });

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(15.25));
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        });

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        let count = new_processor.load_from_csv(path)?;
        
        assert_eq!(count, 1);
        assert_eq!(new_processor.records[0].name, "Test");
        
        Ok(())
    }
}