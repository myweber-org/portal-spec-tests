use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            headers: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_line) = lines.next() {
            self.headers = header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == self.headers.len() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let mut result = Vec::new();
        
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            for record in &self.records {
                if record.get(col_index).map_or(false, |v| v == value) {
                    result.push(record.clone());
                }
            }
        }
        
        result
    }

    pub fn get_column_stats(&self, column_name: &str) -> Option<(usize, usize)> {
        self.headers.iter().position(|h| h == column_name).map(|col_index| {
            let mut min_len = usize::MAX;
            let mut max_len = 0;
            
            for record in &self.records {
                if let Some(value) = record.get(col_index) {
                    let len = value.len();
                    min_len = min_len.min(len);
                    max_len = max_len.max(len);
                }
            }
            
            (min_len, max_len)
        })
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn header_count(&self) -> usize {
        self.headers.len()
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
        writeln!(temp_file, "Charlie,30,Paris").unwrap();
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.header_count(), 3);
        
        let filtered = processor.filter_by_column("age", "30");
        assert_eq!(filtered.len(), 2);
        
        let stats = processor.get_column_stats("name").unwrap();
        assert_eq!(stats, (5, 7));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
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
        
        let mut data = Vec::new();
        for line in lines {
            let line = line?;
            let row: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if row.len() == headers.len() {
                data.push(row);
            }
        }
        
        Ok(CsvProcessor { data, headers })
    }
    
    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data
            .iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }
    
    pub fn aggregate_column(&self, column_index: usize, operation: &str) -> Result<f64, String> {
        if column_index >= self.headers.len() {
            return Err("Column index out of bounds".to_string());
        }
        
        let values: Vec<f64> = self.data
            .iter()
            .filter_map(|row| row[column_index].parse::<f64>().ok())
            .collect();
        
        if values.is_empty() {
            return Err("No valid numeric values found".to_string());
        }
        
        match operation {
            "sum" => Ok(values.iter().sum()),
            "avg" => Ok(values.iter().sum::<f64>() / values.len() as f64),
            "max" => Ok(values
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b))),
            "min" => Ok(values
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b))),
            _ => Err(format!("Unsupported operation: {}", operation)),
        }
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
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
        writeln!(temp_file, "2,item_b,20.3").unwrap();
        writeln!(temp_file, "3,item_c,15.7").unwrap();
        
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.row_count(), 3);
        assert_eq!(processor.column_count(), 3);
        
        let filtered = processor.filter_rows(|row| row[1].contains("item_b"));
        assert_eq!(filtered.len(), 1);
        
        let sum = processor.aggregate_column(2, "sum").unwrap();
        assert!((sum - 46.5).abs() < 0.001);
        
        let avg = processor.aggregate_column(2, "avg").unwrap();
        assert!((avg - 15.5).abs() < 0.001);
    }
}