
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
    pub fn new(id: u32, name: String, value: f64, category: String) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }

        Ok(Self {
            id,
            name,
            value,
            category,
        })
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_number).into());
            }

            let id = parts[0].parse::<u32>()
                .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
            
            let name = parts[1].trim().to_string();
            
            let value = parts[2].parse::<f64>()
                .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
            
            let category = parts[3].trim().to_string();

            let record = CsvRecord::new(id, name, value, category)
                .map_err(|e| format!("Validation error at line {}: {}", line_number, e))?;
            
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn apply_transformation(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform_value(multiplier);
        }
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_creation() {
        let record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(record.is_ok());
        
        let invalid_record = CsvRecord::new(2, "".to_string(), 50.0, "B".to_string());
        assert!(invalid_record.is_err());
    }

    #[test]
    fn test_csv_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item1,100.0,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,300.0,CategoryA").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.get_records().len(), 3);
        
        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 2);
        
        let total = processor.calculate_total_value();
        assert_eq!(total, 600.0);
        
        processor.apply_transformation(2.0);
        let new_total = processor.calculate_total_value();
        assert_eq!(new_total, 1200.0);
    }
}