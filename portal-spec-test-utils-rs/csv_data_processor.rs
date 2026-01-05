use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    category: parts[1].to_string(),
                    value: parts[2].parse()?,
                    active: parts[3].parse().unwrap_or(false),
                };
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn aggregate_by_category(&self) -> HashMap<String, f64> {
        let mut aggregates = HashMap::new();

        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value;
        }

        aggregates
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "id,category,value,active\n1,electronics,250.5,true\n2,furniture,150.0,false\n3,electronics,75.25,true\n4,clothing,45.99,true"
        )
        .unwrap();
        temp_file
    }

    #[test]
    fn test_load_and_filter() {
        let temp_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        
        let count = processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 4);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 3);
    }

    #[test]
    fn test_aggregation() {
        let temp_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.get("electronics"), Some(&325.75));
        
        let average = processor.calculate_average();
        assert!((average - 130.435).abs() < 0.001);
    }
}