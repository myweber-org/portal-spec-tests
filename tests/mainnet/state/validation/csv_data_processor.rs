
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    timestamp: String,
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

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
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

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            total_records: self.records.len(),
            average_value: self.calculate_average(),
            unique_categories: self.get_unique_categories().len(),
        }
    }

    fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .records
            .iter()
            .map(|record| record.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

#[derive(Debug)]
pub struct DataSummary {
    total_records: usize,
    average_value: Option<f64>,
    unique_categories: usize,
}

impl DataSummary {
    pub fn display(&self) {
        println!("Data Summary:");
        println!("  Total Records: {}", self.total_records);
        match self.average_value {
            Some(avg) => println!("  Average Value: {:.2}", avg),
            None => println!("  Average Value: N/A"),
        }
        println!("  Unique Categories: {}", self.unique_categories);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,timestamp").unwrap();
        writeln!(temp_file, "1,electronics,299.99,2024-01-15").unwrap();
        writeln!(temp_file, "2,clothing,49.95,2024-01-16").unwrap();
        writeln!(temp_file, "3,electronics,599.99,2024-01-17").unwrap();
        writeln!(temp_file, "4,books,19.99,2024-01-18").unwrap();
        temp_file
    }

    #[test]
    fn test_load_and_filter() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap())
            .expect("Failed to load CSV");

        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 239.98).abs() < 0.01);
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert!(processor.calculate_average().is_none());
        assert!(processor.find_max_value().is_none());
        assert_eq!(processor.get_summary().total_records, 0);
    }
}