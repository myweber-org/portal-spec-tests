use std::collections::HashSet;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() && self.value.is_finite()
    }
}

pub fn deduplicate_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    let mut seen_ids = HashSet::new();
    let mut unique_records = Vec::new();

    for record in records {
        if seen_ids.insert(record.id) {
            unique_records.push(record);
        }
    }

    unique_records
}

pub fn validate_records(records: &[DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut valid_records = Vec::new();

    for record in records {
        if record.is_valid() {
            valid_records.push(record.clone());
        } else {
            return Err(format!("Invalid record found with id: {}", record.id).into());
        }
    }

    Ok(valid_records)
}

pub fn clean_data(mut records: Vec<DataRecord>) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    records = deduplicate_records(records);
    validate_records(&records)?;
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let records = vec![
            DataRecord { id: 1, value: 10.5, category: "A".to_string() },
            DataRecord { id: 1, value: 20.5, category: "B".to_string() },
            DataRecord { id: 2, value: 30.5, category: "C".to_string() },
        ];

        let deduped = deduplicate_records(records);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_validation() {
        let valid_record = DataRecord { id: 1, value: 10.5, category: "A".to_string() };
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord { id: 2, value: f64::NAN, category: "B".to_string() };
        assert!(!invalid_record.is_valid());
    }
}