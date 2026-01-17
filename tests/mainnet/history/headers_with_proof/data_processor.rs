use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data_file(file_path: &str, min_value: f64) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut filtered_records = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            filtered_records.push(record);
        }
    }
    
    filtered_records.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
    
    Ok(filtered_records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_process_data_file() {
        let csv_data = "id,name,value,active\n1,ItemA,15.5,true\n2,ItemB,8.2,false\n3,ItemC,22.1,true\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = process_data_file(temp_file.path().to_str().unwrap(), 10.0);
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemC");
        assert_eq!(records[1].name, "ItemA");
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
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
            let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
            self.data.push(row);
        }
        
        Ok(())
    }

    pub fn validate_data(&self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        
        let header_len = self.data[0].len();
        for row in &self.data[1..] {
            if row.len() != header_len {
                return false;
            }
        }
        
        true
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
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.row_count(), 3);
        assert_eq!(processor.column_count(), 3);
    }

    #[test]
    fn test_data_validation() {
        let mut processor = DataProcessor::new();
        processor.data = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string()],
        ];
        
        assert!(!processor.validate_data());
    }

    #[test]
    fn test_get_column() {
        let mut processor = DataProcessor::new();
        processor.data = vec![
            vec!["name".to_string(), "age".to_string()],
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        
        let column = processor.get_column(1).unwrap();
        assert_eq!(column, vec!["age", "30", "25"]);
    }
}