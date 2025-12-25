use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if fields.len() < 2 {
                return Err(format!("Invalid data format at line {}", index + 1).into());
            }
            
            records.push(fields);
        }

        if records.is_empty() {
            return Err("No data found in file".into());
        }

        Ok(records)
    }

    pub fn validate_numeric_fields(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();
        
        for (row_index, record) in data.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds at row {}", column_index, row_index + 1).into());
            }
            
            match record[column_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Non-numeric value at row {} column {}", row_index + 1, column_index).into()),
            }
        }
        
        Ok(numeric_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_validate_numeric_fields() {
        let data = vec![
            vec!["Alice".to_string(), "30.5".to_string()],
            vec!["Bob".to_string(), "25.0".to_string()],
        ];
        
        let processor = DataProcessor::new("dummy.csv");
        let numeric_values = processor.validate_numeric_fields(&data, 1).unwrap();
        
        assert_eq!(numeric_values, vec![30.5, 25.0]);
    }
}