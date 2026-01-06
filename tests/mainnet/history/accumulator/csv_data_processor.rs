
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub timestamp: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
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

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<DataRecord>> {
        let mut groups = std::collections::HashMap::new();

        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }

        groups
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            total_records: self.records.len(),
            average_value: self.calculate_average(),
            categories_count: self.group_by_category().len(),
        }
    }
}

#[derive(Debug)]
pub struct DataSummary {
    pub total_records: usize,
    pub average_value: Option<f64>,
    pub categories_count: usize,
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,category,value,timestamp").unwrap();
        writeln!(file, "1,electronics,299.99,2023-01-15").unwrap();
        writeln!(file, "2,clothing,49.95,2023-01-16").unwrap();
        writeln!(file, "3,electronics,599.99,2023-01-17").unwrap();
        writeln!(file, "4,books,19.99,2023-01-18").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!(avg > 0.0);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.value, 599.99);
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert!(processor.calculate_average().is_none());
        assert!(processor.find_max_value().is_none());
    }
}