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