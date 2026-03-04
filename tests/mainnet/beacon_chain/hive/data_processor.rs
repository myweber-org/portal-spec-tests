use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, &'static str> {
        if value < 0.0 {
            return Err("Value cannot be negative");
        }
        if category.is_empty() {
            return Err("Category cannot be empty");
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let (id, value, category): (u32, f64, String) = result?;
        match DataRecord::new(id, value, category) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Skipping invalid record: {}", e),
        }
    }

    Ok(records)
}

pub fn process_records(records: &[DataRecord], tax_rate: f64) -> Vec<(u32, f64)> {
    records
        .iter()
        .map(|record| (record.id, record.calculate_tax(tax_rate)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 100.0, "Electronics".to_string());
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_negative_value() {
        let record = DataRecord::new(2, -50.0, "Books".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_tax_calculation() {
        let record = DataRecord::new(3, 200.0, "Furniture".to_string()).unwrap();
        assert_eq!(record.calculate_tax(0.15), 30.0);
    }
}