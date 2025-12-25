use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);
    
    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;
    
    for result in rdr.records() {
        let record = result?;
        if record.iter().all(|field| !field.trim().is_empty()) {
            wtr.write_record(&record)?;
        }
    }
    
    wtr.flush()?;
    Ok(())
}

pub fn clean_csv_from_stdin() -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(io::stdin());
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(io::stdout());
    
    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;
    
    for result in rdr.records() {
        let record = result?;
        if record.iter().all(|field| !field.trim().is_empty()) {
            wtr.write_record(&record)?;
        }
    }
    
    wtr.flush()?;
    Ok(())
}use std::collections::HashSet;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataCleaner {
    records: Vec<DataRecord>,
    seen_ids: HashSet<u32>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen_ids: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        if self.seen_ids.contains(&record.id) {
            return Err("Duplicate record ID".into());
        }

        if record.value.is_nan() || record.value.is_infinite() {
            return Err("Invalid numeric value".into());
        }

        if record.category.trim().is_empty() {
            return Err("Category cannot be empty".into());
        }

        self.seen_ids.insert(record.id);
        self.records.push(record);
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn remove_outliers(&mut self, threshold: f64) -> usize {
        let avg = self.calculate_average().unwrap_or(0.0);
        let original_len = self.records.len();

        self.records.retain(|record| {
            (record.value - avg).abs() <= threshold
        });

        original_len - self.records.len()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.seen_ids.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut cleaner = DataCleaner::new();
        let record = DataRecord {
            id: 1,
            value: 42.5,
            category: "test".to_string(),
        };

        assert!(cleaner.add_record(record).is_ok());
        assert_eq!(cleaner.get_records().len(), 1);
    }

    #[test]
    fn test_duplicate_id_rejection() {
        let mut cleaner = DataCleaner::new();
        let record1 = DataRecord {
            id: 1,
            value: 10.0,
            category: "cat1".to_string(),
        };
        let record2 = DataRecord {
            id: 1,
            value: 20.0,
            category: "cat2".to_string(),
        };

        assert!(cleaner.add_record(record1).is_ok());
        assert!(cleaner.add_record(record2).is_err());
    }

    #[test]
    fn test_filter_records() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record(DataRecord {
            id: 1,
            value: 10.0,
            category: "A".to_string(),
        }).unwrap();
        cleaner.add_record(DataRecord {
            id: 2,
            value: 20.0,
            category: "B".to_string(),
        }).unwrap();
        cleaner.add_record(DataRecord {
            id: 3,
            value: 30.0,
            category: "A".to_string(),
        }).unwrap();

        let filtered = cleaner.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "A"));
    }
}use std::collections::HashSet;

pub fn remove_duplicates<T: Eq + std::hash::Hash + Clone>(input: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in input {
        if seen.insert(item) {
            result.push(item.clone());
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates_integers() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(remove_duplicates(&input), expected);
    }

    #[test]
    fn test_remove_duplicates_strings() {
        let input = vec!["apple", "banana", "apple", "orange", "banana"];
        let expected = vec!["apple", "banana", "orange"];
        assert_eq!(remove_duplicates(&input), expected);
    }

    #[test]
    fn test_remove_duplicates_empty() {
        let input: Vec<i32> = vec![];
        let expected: Vec<i32> = vec![];
        assert_eq!(remove_duplicates(&input), expected);
    }
}