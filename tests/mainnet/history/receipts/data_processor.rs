use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        if let Some(header_line) = lines.next() {
            let headers: Vec<String> = header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            for line in lines {
                let line = line?;
                let values: Vec<String> = line
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                
                let mut record = HashMap::new();
                for (i, header) in headers.iter().enumerate() {
                    if i < values.len() {
                        record.insert(header.clone(), values[i].clone());
                    }
                }
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self, field: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;
        
        for record in &self.records {
            if let Some(value_str) = record.get(field) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    pub fn count_by_field(&self, field: &str) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for record in &self.records {
            if let Some(value) = record.get(field) {
                *counts.entry(value.clone()).or_insert(0) += 1;
            }
        }
        
        counts
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_field_names(&self) -> Vec<String> {
        if let Some(first_record) = self.records.first() {
            first_record.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,85.5").unwrap();
        writeln!(temp_file, "Bob,30,92.0").unwrap();
        writeln!(temp_file, "Charlie,25,78.5").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_csv(file_path);
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let avg_age = processor.calculate_average("age");
        assert_eq!(avg_age, Some(26.666666666666668));
        
        let age_counts = processor.count_by_field("age");
        assert_eq!(age_counts.get("25"), Some(&2));
        assert_eq!(age_counts.get("30"), Some(&1));
        
        let filtered = processor.filter_records(|record| {
            record.get("age") == Some(&"25".to_string())
        });
        assert_eq!(filtered.len(), 2);
        
        let field_names = processor.get_field_names();
        assert_eq!(field_names, vec!["name", "age", "score"]);
    }
}