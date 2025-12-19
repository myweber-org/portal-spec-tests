
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers = match lines.next() {
            Some(Ok(header_line)) => header_line.split(',').map(|s| s.trim().to_string()).collect(),
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
    
    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };
        
        self.records.iter()
            .filter(|record| record.get(column_index).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Err("Column not found".into()),
        };
        
        let mut result = HashMap::new();
        for record in &self.records {
            if let Some(value_str) = record.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    let key = record.get(0).cloned().unwrap_or_default();
                    *result.entry(key).or_insert(0.0) += value;
                }
            }
        }
        
        Ok(result)
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
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,item_a,10.5").unwrap();
        writeln!(temp_file, "2,item_b,15.3").unwrap();
        writeln!(temp_file, "3,item_a,5.2").unwrap();
        
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.get_record_count(), 3);
        
        let filtered = processor.filter_by_column("name", "item_a");
        assert_eq!(filtered.len(), 2);
        
        let aggregated = processor.aggregate_numeric_column("value").unwrap();
        assert_eq!(aggregated.get("1"), Some(&10.5));
    }
}