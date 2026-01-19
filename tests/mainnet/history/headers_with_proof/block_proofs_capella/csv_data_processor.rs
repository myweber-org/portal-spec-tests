
use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, name: String, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            name,
            category,
            value,
            active,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn category(&self) -> &str {
        &self.category
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        let mut records = Vec::new();

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            records.push(record);
        }

        Ok(Self { records })
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category() == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.is_active())
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value()).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    pub fn export_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn remove_inactive(&mut self) {
        self.records.retain(|record| record.is_active());
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_operations() {
        let records = vec![
            DataRecord::new(1, "Item1".to_string(), "A".to_string(), 100.0, true),
            DataRecord::new(2, "Item2".to_string(), "B".to_string(), 200.0, false),
            DataRecord::new(3, "Item3".to_string(), "A".to_string(), 150.0, true),
        ];

        let mut processor = DataProcessor { records };

        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.filter_by_category("A").len(), 2);
        assert_eq!(processor.filter_active().len(), 2);
        assert_eq!(processor.calculate_total_value(), 450.0);
        assert_eq!(processor.calculate_average_value(), 150.0);

        processor.remove_inactive();
        assert_eq!(processor.count_records(), 2);

        let temp_file = NamedTempFile::new().unwrap();
        let export_result = processor.export_to_csv(temp_file.path().to_str().unwrap());
        assert!(export_result.is_ok());
    }
}