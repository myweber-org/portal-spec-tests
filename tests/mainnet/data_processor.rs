
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<Vec<String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            self.data.push(row);
        }
        
        Ok(())
    }

    pub fn validate_data(&self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        
        let first_row_len = self.data[0].len();
        self.data.iter().all(|row| row.len() == first_row_len)
    }

    pub fn get_column(&self, index: usize) -> Option<Vec<String>> {
        if index >= self.data.get(0)?.len() {
            return None;
        }
        
        let mut column = Vec::new();
        for row in &self.data {
            if let Some(value) = row.get(index) {
                column.push(value.clone());
            }
        }
        
        Some(column)
    }

    pub fn row_count(&self) -> usize {
        self.data.len()
    }

    pub fn column_count(&self) -> usize {
        self.data.get(0).map_or(0, |row| row.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(processor.validate_data());
        assert_eq!(processor.row_count(), 2);
        assert_eq!(processor.column_count(), 3);
        
        let names = processor.get_column(0).unwrap();
        assert_eq!(names, vec!["Alice", "Bob"]);
    }
}