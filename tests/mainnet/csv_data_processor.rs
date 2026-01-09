use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub columns: Vec<String>,
    pub values: HashMap<String, String>,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
    headers: Vec<String>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers_line = lines.next().ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line.split(',').map(|s| s.trim().to_string()).collect();

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if values.len() != headers.len() {
                continue;
            }

            let mut record_map = HashMap::new();
            for (i, header) in headers.iter().enumerate() {
                record_map.insert(header.clone(), values[i].clone());
            }

            records.push(CsvRecord {
                columns: headers.clone(),
                values: record_map,
            });
        }

        Ok(CsvProcessor { records, headers })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| {
                record.values
                    .get(column_name)
                    .map(|value| predicate(value))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, String> {
        let mut total = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.values.get(column_name) {
                if let Ok(value) = value_str.parse::<f64>() {
                    total += value;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Err(format!("No valid numeric values found in column '{}'", column_name));
        }

        Ok(total / count as f64)
    }

    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        let mut unique_values = std::collections::HashSet::new();
        
        for record in &self.records {
            if let Some(value) = record.values.get(column_name) {
                unique_values.insert(value.clone());
            }
        }

        unique_values.into_iter().collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_names(&self) -> &[String] {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();
        writeln!(temp_file, "Diana,30,55000").unwrap();
        temp_file
    }

    #[test]
    fn test_csv_loading() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.record_count(), 4);
        assert_eq!(processor.column_names(), &["name", "age", "salary"]);
    }

    #[test]
    fn test_filter_by_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("age", |age| age == "30");
        assert_eq!(filtered.len(), 2);
        
        let names: Vec<String> = filtered
            .iter()
            .map(|r| r.values.get("name").unwrap().clone())
            .collect();
        assert!(names.contains(&"Alice".to_string()));
        assert!(names.contains(&"Diana".to_string()));
    }

    #[test]
    fn test_aggregate_numeric_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let avg_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert!((avg_salary - 52500.0).abs() < 0.001);
    }

    #[test]
    fn test_get_unique_values() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let unique_ages = processor.get_unique_values("age");
        assert_eq!(unique_ages.len(), 3);
        assert!(unique_ages.contains(&"30".to_string()));
        assert!(unique_ages.contains(&"25".to_string()));
        assert!(unique_ages.contains(&"35".to_string()));
    }
}