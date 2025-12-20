
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_active_records(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            *counts.entry(record.category.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,category,value,active").unwrap();
        writeln!(file, "1,electronics,250.50,true").unwrap();
        writeln!(file, "2,books,45.99,false").unwrap();
        writeln!(file, "3,electronics,120.75,true").unwrap();
        writeln!(file, "4,clothing,89.99,true").unwrap();
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
        
        let active_records = processor.get_active_records();
        assert_eq!(active_records.len(), 3);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 1);
        
        let counts = processor.count_by_category();
        assert_eq!(counts.get("electronics"), Some(&2));
    }
}