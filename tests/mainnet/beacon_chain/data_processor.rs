
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_headers: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        DataProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(self.has_headers)
            .from_reader(file);

        let mut records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            records.push(fields);
        }

        Ok(records)
    }

    pub fn validate_data(&self, data: &[Vec<String>], expected_columns: usize) -> Result<(), String> {
        for (index, row) in data.iter().enumerate() {
            if row.len() != expected_columns {
                return Err(format!(
                    "Row {} has {} columns, expected {}",
                    index + 1,
                    row.len(),
                    expected_columns
                ));
            }
        }
        Ok(())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<String>, String> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut column_data = Vec::with_capacity(data.len());
        for (row_index, row) in data.iter().enumerate() {
            if column_index >= row.len() {
                return Err(format!(
                    "Column index {} out of bounds for row {}",
                    column_index,
                    row_index + 1
                ));
            }
            column_data.push(row[column_index].clone());
        }
        Ok(column_data)
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

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path());
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validation() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_data(&data, 2).is_ok());
        assert!(processor.validate_data(&data, 3).is_err());
    }

    #[test]
    fn test_column_extraction() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 0).unwrap();
        assert_eq!(column, vec!["a", "c"]);
        
        let result = processor.extract_column(&data, 5);
        assert!(result.is_err());
    }
}