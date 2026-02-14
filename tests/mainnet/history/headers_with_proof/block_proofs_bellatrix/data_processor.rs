use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];
        
        assert!(processor.validate_record(&valid_record, 2));
        assert!(!processor.validate_record(&invalid_record, 2));
    }

    #[test]
    fn test_extract_column() {
        let processor = DataProcessor::new(',', false);
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let column = processor.extract_column(&data, 1);
        assert_eq!(column, vec!["b".to_string(), "d".to_string()]);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    MissingField,
    CategoryNotFound,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::CategoryNotFound => write!(f, "Category not found in mapping"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_mapping: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_mapping: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    pub fn add_category_mapping(&mut self, from: String, to: String) {
        self.category_mapping.insert(from, to);
    }

    pub fn process_records(&mut self) -> Result<Vec<DataRecord>, DataError> {
        let mut processed = Vec::with_capacity(self.records.len());
        
        for record in &self.records {
            let transformed = self.transform_record(record)?;
            processed.push(transformed);
        }
        
        Ok(processed)
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }
        
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let average = total / count;
        
        let max_value = self.records.iter()
            .map(|r| r.value)
            .fold(f64::MIN, |a, b| a.max(b));
        
        let min_value = self.records.iter()
            .map(|r| r.value)
            .fold(f64::MAX, |a, b| a.min(b));
        
        stats.insert("total".to_string(), total);
        stats.insert("average".to_string(), average);
        stats.insert("max".to_string(), max_value);
        stats.insert("min".to_string(), min_value);
        stats.insert("count".to_string(), count);
        
        stats
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }
        
        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
        }
        
        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, DataError> {
        let normalized_category = if let Some(mapped) = self.category_mapping.get(&record.category) {
            mapped.clone()
        } else if record.category.is_empty() {
            return Err(DataError::CategoryNotFound);
        } else {
            record.category.clone()
        };
        
        let normalized_name = record.name.trim().to_string();
        let adjusted_value = if record.value < 0.0 {
            0.0
        } else {
            record.value
        };
        
        Ok(DataRecord {
            id: record.id,
            name: normalized_name,
            value: adjusted_value,
            category: normalized_category,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.calculate_statistics()["count"], 1.0);
    }

    #[test]
    fn test_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: f64::NAN,
            category: "".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_mapping() {
        let mut processor = DataProcessor::new();
        processor.add_category_mapping("OLD".to_string(), "NEW".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "OLD".to_string(),
        };
        
        processor.add_record(record).unwrap();
        let processed = processor.process_records().unwrap();
        
        assert_eq!(processed[0].category, "NEW");
    }
}