use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err("File does not exist".into());
        }

        let mut rdr = Reader::from_path(file_path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&Record> = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_data = "id,name,value,category\n1,Test1,10.5,A\n2,Test2,-5.0,B\n3,,15.0,A";
        let mut file = File::create("test_data.csv").unwrap();
        file.write_all(test_data.as_bytes()).unwrap();

        let result = processor.load_from_csv("test_data.csv");
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);

        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 2);

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(12.75));

        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);

        std::fs::remove_file("test_data.csv").unwrap();
    }
}