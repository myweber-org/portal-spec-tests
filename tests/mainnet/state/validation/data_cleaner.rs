use csv::{Reader, Writer};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct RawRecord {
    id: String,
    value: String,
    category: String,
}

#[derive(Debug)]
struct CleanRecord {
    id: u32,
    value: f64,
    category: String,
}

impl CleanRecord {
    fn from_raw(raw: RawRecord) -> Result<Self, Box<dyn Error>> {
        let id = raw.id.parse::<u32>()?;
        let value = raw.value.parse::<f64>()?;
        let category = raw.category.trim().to_lowercase();

        if value < 0.0 {
            return Err("Negative values not allowed".into());
        }

        Ok(CleanRecord {
            id,
            value,
            category,
        })
    }
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_writer(File::create(output_path)?);

    writer.write_record(&["id", "value", "category"])?;

    for result in reader.deserialize() {
        let raw: RawRecord = result?;
        
        match CleanRecord::from_raw(raw) {
            Ok(clean) => {
                writer.write_record(&[
                    clean.id.to_string(),
                    clean.value.to_string(),
                    clean.category,
                ])?;
            }
            Err(e) => eprintln!("Skipping record: {}", e),
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_csv("input.csv", "output.csv")?;
    println!("Data cleaning completed successfully");
    Ok(())
}use std::collections::HashSet;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        Self {
            id,
            value,
            category: category.to_string(),
        }
    }
}

pub struct DataCleaner;

impl DataCleaner {
    pub fn deduplicate_records(records: &[DataRecord]) -> Vec<DataRecord> {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();

        for record in records {
            if seen.insert(record.id) {
                unique_records.push(record.clone());
            }
        }

        unique_records
    }

    pub fn validate_record(record: &DataRecord) -> Result<(), Box<dyn Error>> {
        if record.id == 0 {
            return Err("ID cannot be zero".into());
        }

        if record.value.is_nan() || record.value.is_infinite() {
            return Err("Value must be a finite number".into());
        }

        if record.category.trim().is_empty() {
            return Err("Category cannot be empty".into());
        }

        Ok(())
    }

    pub fn filter_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<DataRecord> {
        records
            .iter()
            .filter(|r| r.value >= threshold)
            .cloned()
            .collect()
    }

    pub fn normalize_values(records: &mut [DataRecord]) {
        if records.is_empty() {
            return;
        }

        let max_value = records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        if max_value > 0.0 {
            for record in records.iter_mut() {
                record.value /= max_value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let records = vec![
            DataRecord::new(1, 10.0, "A"),
            DataRecord::new(2, 20.0, "B"),
            DataRecord::new(1, 30.0, "C"),
        ];

        let deduped = DataCleaner::deduplicate_records(&records);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_validation() {
        let valid = DataRecord::new(1, 5.0, "Test");
        assert!(DataCleaner::validate_record(&valid).is_ok());

        let invalid = DataRecord::new(0, 5.0, "Test");
        assert!(DataCleaner::validate_record(&invalid).is_err());
    }

    #[test]
    fn test_filter_threshold() {
        let records = vec![
            DataRecord::new(1, 5.0, "A"),
            DataRecord::new(2, 15.0, "B"),
            DataRecord::new(3, 25.0, "C"),
        ];

        let filtered = DataCleaner::filter_by_threshold(&records, 10.0);
        assert_eq!(filtered.len(), 2);
    }
}