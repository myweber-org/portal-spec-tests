
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|record| record.category.clone())
            .collect();

        categories.sort();
        categories.dedup();
        categories
    }

    pub fn validate_records(&self) -> Vec<u32> {
        self.records
            .iter()
            .filter(|record| record.value.is_nan() || record.value.is_infinite())
            .map(|record| record.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,42.5,TypeA").unwrap();
        writeln!(temp_file, "2,18.3,TypeB").unwrap();
        writeln!(temp_file, "3,75.0,TypeA").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);

        let filtered = processor.filter_by_threshold(20.0);
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 45.266).abs() < 0.001);

        let categories = processor.get_unique_categories();
        assert_eq!(categories, vec!["TypeA", "TypeB"]);
    }
}