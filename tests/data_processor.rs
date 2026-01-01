use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let mut records = Vec::new();
    
    for result in rdr.deserialize() {
        let record: DataRecord = result?;
        records.push(record);
    }
    
    Ok(records)
}

pub fn validate_records(records: &[DataRecord]) -> Result<(), String> {
    for record in records {
        if record.id == 0 {
            return Err(format!("Invalid ID found: {}", record.id));
        }
        
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(format!("Invalid value found for ID {}: {}", record.id, record.value));
        }
        
        if record.category.is_empty() {
            return Err(format!("Empty category for ID {}", record.id));
        }
    }
    
    Ok(())
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation() {
        let valid_record = DataRecord {
            id: 1,
            value: 42.5,
            category: String::from("test"),
        };
        
        let invalid_record = DataRecord {
            id: 0,
            value: f64::NAN,
            category: String::new(),
        };
        
        let records = vec![valid_record, invalid_record];
        assert!(validate_records(&records).is_err());
    }
    
    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, category: String::from("A") },
            DataRecord { id: 2, value: 20.0, category: String::from("B") },
            DataRecord { id: 3, value: 30.0, category: String::from("C") },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}