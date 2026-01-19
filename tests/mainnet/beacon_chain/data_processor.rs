
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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
    }
}

pub fn process_csv_file(file_path: &Path) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].trim().to_string();

        match DataRecord::new(id, value, category) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

pub fn aggregate_values(records: &[DataRecord]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / count;
    let max = records.iter().map(|r| r.value).fold(f64::MIN, f64::max);

    (sum, avg, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 100.5, "electronics".to_string());
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.5);
        assert_eq!(record.category, "electronics");
    }

    #[test]
    fn test_invalid_record_creation() {
        let record = DataRecord::new(2, -10.0, "books".to_string());
        assert!(record.is_err());

        let record = DataRecord::new(3, 50.0, "".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_calculate_tax() {
        let record = DataRecord::new(1, 200.0, "clothing".to_string()).unwrap();
        let tax = record.calculate_tax(0.15);
        assert_eq!(tax, 30.0);
    }

    #[test]
    fn test_process_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,100.5,electronics").unwrap();
        writeln!(temp_file, "2,75.3,books").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,250.0,clothing").unwrap();

        let records = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].category, "books");
        assert_eq!(records[2].value, 250.0);
    }

    #[test]
    fn test_aggregate_values() {
        let records = vec![
            DataRecord::new(1, 100.0, "cat1".to_string()).unwrap(),
            DataRecord::new(2, 200.0, "cat2".to_string()).unwrap(),
            DataRecord::new(3, 300.0, "cat3".to_string()).unwrap(),
        ];

        let (sum, avg, max) = aggregate_values(&records);
        assert_eq!(sum, 600.0);
        assert_eq!(avg, 200.0);
        assert_eq!(max, 300.0);
    }

    #[test]
    fn test_aggregate_empty() {
        let records: Vec<DataRecord> = vec![];
        let (sum, avg, max) = aggregate_values(&records);
        assert_eq!(sum, 0.0);
        assert_eq!(avg, 0.0);
        assert_eq!(max, 0.0);
    }
}