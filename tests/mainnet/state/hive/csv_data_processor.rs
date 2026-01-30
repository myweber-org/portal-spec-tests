use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
    pub values: HashMap<String, String>,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
    headers: Vec<String>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            headers: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_line) = lines.next() {
            let header_line = header_line?;
            self.headers = header_line.split(',').map(|s| s.trim().to_string()).collect();
        }

        for line_result in lines {
            let line = line_result?;
            let values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if values.len() != self.headers.len() {
                continue;
            }

            let mut record_map = HashMap::new();
            for (i, header) in self.headers.iter().enumerate() {
                record_map.insert(header.clone(), values[i].clone());
            }

            let record = CsvRecord {
                columns: self.headers.clone(),
                values: record_map,
            };

            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_column(&self, column_name: &str, filter_value: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| {
                record.values.get(column_name)
                    .map(|val| val == filter_value)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.values.get(column_name) {
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

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,New York").unwrap();

        let file_path = temp_file.path().to_str().unwrap();
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(file_path).unwrap();

        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.get_headers(), &vec!["name".to_string(), "age".to_string(), "city".to_string()]);

        let ny_records = processor.filter_by_column("city", "New York");
        assert_eq!(ny_records.len(), 2);

        let avg_age = processor.aggregate_numeric_column("age");
        assert!(avg_age.is_some());
        assert!((avg_age.unwrap() - 30.0).abs() < 0.001);
    }
}