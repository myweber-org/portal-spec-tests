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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn get_column_summary(&self, column_name: &str) -> Option<(usize, Vec<String>)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let mut unique_values = Vec::new();
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                if !unique_values.contains(value) {
                    unique_values.push(value.clone());
                }
            }
        }
        
        Some((unique_values.len(), unique_values))
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,department").unwrap();
        writeln!(file, "1,Alice,Engineering").unwrap();
        writeln!(file, "2,Bob,Marketing").unwrap();
        writeln!(file, "3,Charlie,Engineering").unwrap();
        writeln!(file, "4,Diana,Sales").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.headers(), vec!["id", "name", "department"]);
        assert_eq!(processor.record_count(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let first_record = &engineering_records[0];
        assert_eq!(first_record[1], "Alice");
    }

    #[test]
    fn test_column_summary() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let summary = processor.get_column_summary("department").unwrap();
        assert_eq!(summary.0, 3);
        assert!(summary.1.contains(&"Engineering".to_string()));
        assert!(summary.1.contains(&"Marketing".to_string()));
        assert!(summary.1.contains(&"Sales".to_string()));
    }
}