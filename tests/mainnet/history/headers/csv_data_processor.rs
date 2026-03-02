use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let id = parts[0].parse::<u32>().unwrap_or_default();
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>().unwrap_or_default();
        let category = parts[3].to_string();

        let record = Record::new(id, name, value, category);
        if record.is_valid() {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn process_records(records: &mut [Record], multiplier: f64) {
    for record in records.iter_mut() {
        record.transform_value(multiplier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = Record::new(1, "Test".to_string(), 10.0, "A".to_string());
        record.transform_value(2.5);
        assert_eq!(record.value, 25.0);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, "X".to_string()),
            Record::new(2, "B".to_string(), 20.0, "Y".to_string()),
            Record::new(3, "C".to_string(), 30.0, "X".to_string()),
        ];

        let filtered = filter_by_category(&records, "X");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_total() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, "X".to_string()),
            Record::new(2, "B".to_string(), 20.0, "Y".to_string()),
        ];

        let total = calculate_total_value(&records);
        assert_eq!(total, 30.0);
    }
}