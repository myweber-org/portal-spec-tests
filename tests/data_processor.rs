
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        
        for line in reader.lines() {
            let line_content = line?;
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }
        
        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No valid records found".into());
        }
        
        let header_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != header_len {
                return Err(format!("Record {} has inconsistent field count", i).into());
            }
        }
        
        Ok(())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to process".into());
        }
        
        if column_index >= records[0].len() {
            return Err("Column index out of bounds".into());
        }
        
        let column_data: Vec<String> = records
            .iter()
            .map(|record| record[column_index].clone())
            .collect();
        
        Ok(column_data)
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let records = processor.process().unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[1][0], "John");
        assert_eq!(records[2][1], "25");
        
        processor.validate_records(&records).unwrap();
        
        let names = processor.extract_column(&records, 0).unwrap();
        assert_eq!(names, vec!["name", "John", "Alice"]);
    }
}