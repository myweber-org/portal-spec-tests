
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            category,
            value,
            active,
        }
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn max_value(&self) -> Option<f64> {
        self.records.iter().map(|record| record.value).reduce(f64::max)
    }

    pub fn min_value(&self) -> Option<f64> {
        self.records.iter().map(|record| record.value).reduce(f64::min)
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::Reader::from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.add_record(record);
        }

        Ok(())
    }

    pub fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = csv::Writer::from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn summary(&self) -> String {
        format!(
            "Records: {}, Active: {}, Avg Value: {:.2}, Max: {:.2}, Min: {:.2}",
            self.records.len(),
            self.filter_active().len(),
            self.average_value().unwrap_or(0.0),
            self.max_value().unwrap_or(0.0),
            self.min_value().unwrap_or(0.0)
        )
    }
}

pub fn process_data_file(input_path: &str, output_path: &str) -> Result<String, Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    processor.load_from_csv(input_path)?;

    let active_records = processor.filter_active();
    let tech_records = processor.filter_by_category("Technology");

    let summary = processor.summary();
    processor.save_to_csv(output_path)?;

    Ok(format!(
        "Processed {} records. Active: {}, Technology: {}. Summary: {}",
        processor.records.len(),
        active_records.len(),
        tech_records.len(),
        summary
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        processor.add_record(DataRecord::new(1, "Technology".to_string(), 100.0, true));
        processor.add_record(DataRecord::new(2, "Finance".to_string(), 200.0, true));
        processor.add_record(DataRecord::new(3, "Technology".to_string(), 150.0, false));

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.filter_by_category("Technology").len(), 2);
        assert_eq!(processor.filter_active().len(), 2);
        assert_eq!(processor.average_value(), Some(150.0));
        assert_eq!(processor.max_value(), Some(200.0));
        assert_eq!(processor.min_value(), Some(100.0));
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.records.len(), 0);
        assert_eq!(processor.average_value(), None);
        assert_eq!(processor.max_value(), None);
        assert_eq!(processor.min_value(), None);
    }
}