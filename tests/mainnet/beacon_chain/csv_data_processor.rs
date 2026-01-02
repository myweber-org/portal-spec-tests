
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn read_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.is_valid() {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn write_csv<P: AsRef<Path>>(path: P, records: &[Record]) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut wtr = csv::Writer::from_writer(writer);

    for record in records {
        wtr.serialize(record)?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn process_records(records: &mut [Record], multiplier: f64) {
    for record in records.iter_mut() {
        if record.is_valid() {
            record.transform(multiplier);
        }
    }
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.category == category && r.is_valid())
        .collect()
}

pub fn calculate_total_value(records: &[Record]) -> f64 {
    records
        .iter()
        .filter(|r| r.is_valid())
        .map(|r| r.value)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.0, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = Record::new(1, "Test".to_string(), 10.0, "A".to_string());
        record.transform(2.0);
        assert_eq!(record.value, 20.0);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let records = vec![
            Record::new(1, "Item1".to_string(), 10.0, "CategoryA".to_string()),
            Record::new(2, "Item2".to_string(), 20.0, "CategoryB".to_string()),
        ];

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        write_csv(path, &records)?;
        let loaded_records = read_csv(path)?;

        assert_eq!(loaded_records.len(), 2);
        Ok(())
    }

    #[test]
    fn test_filter_and_calculation() {
        let records = vec![
            Record::new(1, "Item1".to_string(), 10.0, "CategoryA".to_string()),
            Record::new(2, "Item2".to_string(), 20.0, "CategoryA".to_string()),
            Record::new(3, "Item3".to_string(), 30.0, "CategoryB".to_string()),
        ];

        let filtered = filter_by_category(&records, "CategoryA");
        assert_eq!(filtered.len(), 2);

        let total = calculate_total_value(&records);
        assert_eq!(total, 60.0);
    }
}