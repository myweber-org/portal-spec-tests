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

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        if records.is_empty() {
            return Err("No valid data found in file".into());
        }

        Ok(records)
    }

    pub fn validate_numeric_fields(&self, records: &[Vec<String>], field_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();

        for (row_num, record) in records.iter().enumerate() {
            if field_index >= record.len() {
                return Err(format!("Field index {} out of bounds on row {}", field_index, row_num + 1).into());
            }

            match record[field_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at row {}: '{}'", row_num + 1, record[field_index]).into()),
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
        let records = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];

        let processor = DataProcessor::new("dummy.csv");
        let numeric_values = processor.validate_numeric_fields(&records, 1).unwrap();

        assert_eq!(numeric_values, vec![30.0, 25.0]);
    }

    #[test]
    fn test_validate_invalid_numeric() {
        let records = vec![
            vec!["Alice".to_string(), "thirty".to_string()],
        ];

        let processor = DataProcessor::new("dummy.csv");
        let result = processor.validate_numeric_fields(&records, 1);

        assert!(result.is_err());
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
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidId => write!(f, "ID must be greater than zero"),
            ProcessingError::EmptyName => write!(f, "Name cannot be empty"),
            ProcessingError::NegativeValue => write!(f, "Value must be non-negative"),
            ProcessingError::DuplicateTag => write!(f, "Tags must be unique"),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, ProcessingError> {
        if id == 0 {
            return Err(ProcessingError::InvalidId);
        }
        
        if name.trim().is_empty() {
            return Err(ProcessingError::EmptyName);
        }
        
        if value < 0.0 {
            return Err(ProcessingError::NegativeValue);
        }
        
        let mut seen_tags = HashMap::new();
        for tag in &tags {
            if seen_tags.contains_key(tag) {
                return Err(ProcessingError::DuplicateTag);
            }
            seen_tags.insert(tag.clone(), true);
        }
        
        Ok(Self {
            id,
            name,
            value,
            tags,
        })
    }
    
    pub fn normalize_value(&mut self, factor: f64) {
        if factor != 0.0 {
            self.value /= factor;
        }
    }
    
    pub fn add_tag(&mut self, tag: String) -> Result<(), ProcessingError> {
        if self.tags.contains(&tag) {
            return Err(ProcessingError::DuplicateTag);
        }
        self.tags.push(tag);
        Ok(())
    }
    
    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value * 100.0;
        let tag_bonus = self.tags.len() as f64 * 5.0;
        base_score + tag_bonus
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<f64, ProcessingError>> {
    records.iter_mut()
        .map(|record| {
            record.normalize_value(10.0);
            if record.value > 1000.0 {
                Err(ProcessingError::NegativeValue)
            } else {
                Ok(record.calculate_score())
            }
        })
        .collect()
}

pub fn filter_records_by_tag(records: &[DataRecord], tag_filter: &str) -> Vec<&DataRecord> {
    records.iter()
        .filter(|record| record.tags.iter().any(|tag| tag.contains(tag_filter)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            42.5,
            vec!["alpha".to_string(), "beta".to_string()]
        ).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Record");
        assert_eq!(record.value, 42.5);
        assert_eq!(record.tags.len(), 2);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(
            0,
            "Test".to_string(),
            10.0,
            vec![]
        );
        
        assert!(matches!(result, Err(ProcessingError::InvalidId)));
    }
    
    #[test]
    fn test_calculate_score() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            5.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        ).unwrap();
        
        let score = record.calculate_score();
        assert_eq!(score, 5.0 * 100.0 + 2.0 * 5.0);
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStatistics>,
}

#[derive(Debug, Clone)]
pub struct CategoryStatistics {
    pub count: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Err(ProcessingError::InvalidData("No records to process".to_string()));
        }

        self.calculate_statistics();
        self.normalize_values()?;
        Ok(())
    }

    pub fn get_category_statistics(&self, category: &str) -> Option<&CategoryStatistics> {
        self.category_stats.get(category)
    }

    pub fn get_all_statistics(&self) -> &HashMap<String, CategoryStatistics> {
        &self.category_stats
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value cannot be negative".to_string(),
            ));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Category cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStatistics {
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.count += 1;
        stats.total_value += record.value;
    }

    fn calculate_statistics(&mut self) {
        for (_, stats) in self.category_stats.iter_mut() {
            if stats.count > 0 {
                stats.average_value = stats.total_value / stats.count as f64;
            }
        }
    }

    fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Ok(());
        }

        let max_value = self.records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        if max_value <= 0.0 {
            return Err(ProcessingError::TransformationError(
                "Cannot normalize with non-positive maximum value".to_string(),
            ));
        }

        for record in &mut self.records {
            record.value = record.value / max_value;
        }

        self.category_stats.clear();
        for record in &self.records {
            self.update_category_stats(record);
        }
        self.calculate_statistics();

        Ok(())
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
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
            name: "Test Record".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_process_records() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 50.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 150.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 75.0,
                category: "B".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        assert!(processor.process_records().is_ok());
        
        let stats_a = processor.get_category_statistics("A").unwrap();
        assert_eq!(stats_a.count, 2);
        
        let stats_b = processor.get_category_statistics("B").unwrap();
        assert_eq!(stats_b.count, 1);
    }

    #[test]
    fn test_filter_by_threshold() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 30.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 70.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 50.0,
                category: "B".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let filtered = processor.filter_by_threshold(50.0);
        assert_eq!(filtered.len(), 2);
    }
}