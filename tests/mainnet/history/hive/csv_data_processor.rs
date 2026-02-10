
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        CsvRecord {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub fn read_csv_file(file_path: &Path) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line_num == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let category = parts[3].to_string();

        let record = CsvRecord::new(id, name, value, category);
        if let Err(e) = record.validate() {
            return Err(format!("Validation error at line {}: {}", line_num + 1, e).into());
        }

        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<CsvRecord> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn transform_records(records: &[CsvRecord]) -> Vec<CsvRecord> {
    records
        .iter()
        .map(|r| {
            let transformed_value = if r.value > 100.0 {
                r.value * 0.9
            } else {
                r.value * 1.1
            };
            CsvRecord::new(r.id, r.name.clone(), transformed_value, r.category.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 50.0, "CategoryA".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord::new(2, "".to_string(), -10.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,100.0,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();

        let records = read_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Item1");
        assert_eq!(records[1].value, 200.0);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "Cat1".to_string()),
            CsvRecord::new(2, "B".to_string(), 20.0, "Cat2".to_string()),
            CsvRecord::new(3, "C".to_string(), 30.0, "Cat1".to_string()),
        ];

        let filtered = filter_by_category(&records, "Cat1");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "Cat1".to_string()),
            CsvRecord::new(2, "B".to_string(), 20.0, "Cat2".to_string()),
            CsvRecord::new(3, "C".to_string(), 30.0, "Cat1".to_string()),
        ];

        let total = calculate_total_value(&records);
        assert_eq!(total, 60.0);
    }

    #[test]
    fn test_transform_records() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 150.0, "Cat1".to_string()),
            CsvRecord::new(2, "B".to_string(), 50.0, "Cat2".to_string()),
        ];

        let transformed = transform_records(&records);
        assert_eq!(transformed[0].value, 135.0);
        assert_eq!(transformed[1].value, 55.0);
    }
}