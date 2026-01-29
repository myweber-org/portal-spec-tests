
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>().unwrap_or(0.0);
            let category = parts[3].to_string();
            
            let record = CsvRecord::new(id, name, value, category);
            if let Err(e) = record.validate() {
                eprintln!("Validation error on line {}: {}", line_num + 1, e);
                continue;
            }
            
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord::new(2, "".to_string(), -10.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,100.0,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,300.0,CategoryA").unwrap();

        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path()).unwrap();

        assert_eq!(processor.get_records().len(), 3);
        assert_eq!(processor.calculate_total_value(), 600.0);
        assert_eq!(processor.get_average_value(), 200.0);

        let category_a_records = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_records.len(), 2);

        processor.transform_values(|v| v * 1.1);
        assert_eq!(processor.calculate_total_value(), 660.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(Ok(line)) => line.split(',').map(|s| s.trim().to_string()).collect(),
            _ => return Err("Empty CSV file".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| predicate(&record[column_index]))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: AggregationOp) -> Result<f64, String> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Err(format!("Column '{}' not found", column_name)),
        };

        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return Err("No valid numeric values found".into());
        }

        match operation {
            AggregationOp::Sum => Ok(numeric_values.iter().sum()),
            AggregationOp::Average => Ok(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            AggregationOp::Max => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap())
                .ok_or("Cannot compute max".into())
                .map(|&v| v),
            AggregationOp::Min => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap())
                .ok_or("Cannot compute min".into())
                .map(|&v| v),
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

pub enum AggregationOp {
    Sum,
    Average,
    Max,
    Min,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary").unwrap();
        writeln!(file, "Alice,30,50000").unwrap();
        writeln!(file, "Bob,25,45000").unwrap();
        writeln!(file, "Charlie,35,60000").unwrap();
        writeln!(file, "Diana,28,55000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), &["name", "age", "salary"]);
        assert_eq!(processor.get_record_count(), 4);
    }

    #[test]
    fn test_filter_by_age() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("age", |age| age.parse::<i32>().unwrap() >= 30);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Charlie");
    }

    #[test]
    fn test_aggregate_salary() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let total_salary = processor.aggregate_numeric_column("salary", AggregationOp::Sum).unwrap();
        assert!((total_salary - 210000.0).abs() < 0.001);
        
        let avg_salary = processor.aggregate_numeric_column("salary", AggregationOp::Average).unwrap();
        assert!((avg_salary - 52500.0).abs() < 0.001);
    }
}