
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(path)?;
        for record in &self.records {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, id: u32, name: String, value: f64, active: bool) {
        self.records.push(Record {
            id,
            name,
            value,
            active,
        });
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter().find(|r| r.id == target_id)
    }

    pub fn validate_records(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for record in &self.records {
            if record.name.is_empty() {
                errors.push(format!("Record {} has empty name", record.id));
            }
            if record.value < 0.0 {
                errors.push(format!("Record {} has negative value", record.id));
            }
        }
        errors
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_and_count() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count(), 0);

        processor.add_record(1, "Test".to_string(), 10.5, true);
        assert_eq!(processor.count(), 1);
    }

    #[test]
    fn test_filter_active() {
        let mut processor = DataProcessor::new();
        processor.add_record(1, "Active".to_string(), 10.0, true);
        processor.add_record(2, "Inactive".to_string(), 20.0, false);

        let active = processor.filter_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_calculate_total() {
        let mut processor = DataProcessor::new();
        processor.add_record(1, "A".to_string(), 10.0, true);
        processor.add_record(2, "B".to_string(), 20.0, true);
        processor.add_record(3, "C".to_string(), 30.0, true);

        assert_eq!(processor.calculate_total(), 60.0);
    }

    #[test]
    fn test_csv_roundtrip() {
        let mut processor = DataProcessor::new();
        processor.add_record(1, "First".to_string(), 100.0, true);
        processor.add_record(2, "Second".to_string(), 200.0, false);

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        processor.save_to_csv(path).unwrap();

        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path).unwrap();

        assert_eq!(new_processor.count(), 2);
        assert_eq!(new_processor.calculate_total(), 300.0);
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_record(1, "".to_string(), -10.0, true);
        processor.add_record(2, "Valid".to_string(), 20.0, true);

        let errors = processor.validate_records();
        assert_eq!(errors.len(), 2);
    }
}