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
        
        let headers: Vec<String> = if let Some(first_line) = lines.next() {
            first_line?.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            return Err("Empty CSV file".into());
        };
        
        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?.split(',').map(|s| s.trim().to_string()).collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }
        
        Ok(CsvProcessor { headers, records })
    }
    
    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        
        if let Some(idx) = column_index {
            self.records.iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Result<f64, String> {
        let column_index = self.headers.iter().position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        let numeric_values: Vec<f64> = self.records.iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .collect();
        
        if numeric_values.is_empty() {
            return Err("No valid numeric values found".into());
        }
        
        match operation {
            "sum" => Ok(numeric_values.iter().sum()),
            "avg" => Ok(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => Ok(numeric_values.iter().fold(f64::MIN, |a, &b| a.max(b))),
            "min" => Ok(numeric_values.iter().fold(f64::MAX, |a, &b| a.min(b))),
            _ => Err(format!("Unsupported operation: {}", operation))
        }
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
    
    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();
        
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.record_count(), 3);
        
        let filtered = processor.filter_by_column("age", |age| age.parse::<i32>().unwrap() > 30);
        assert_eq!(filtered.len(), 1);
        
        let avg_salary = processor.aggregate_numeric_column("salary", "avg").unwrap();
        assert!((avg_salary - 51666.666).abs() < 0.001);
    }
}