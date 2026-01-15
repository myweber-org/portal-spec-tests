
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let valid = parts[3].parse::<bool>().unwrap_or(false);

            let record = DataRecord {
                id,
                value,
                category,
                valid,
            };

            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.valid)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_initialization() {
        let processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
    }

    #[test]
    fn test_load_from_csv() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,CategoryA,true").unwrap();
        writeln!(temp_file, "2,20.3,CategoryB,false").unwrap();
        writeln!(temp_file, "3,15.7,CategoryA,true").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_filter_valid() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(DataRecord {
            id: 1,
            value: 10.5,
            category: "A".to_string(),
            valid: true,
        });
        
        processor.records.push(DataRecord {
            id: 2,
            value: 20.3,
            category: "B".to_string(),
            valid: false,
        });
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 1);
        assert_eq!(valid_records[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "A".to_string(),
            valid: true,
        });
        
        processor.records.push(DataRecord {
            id: 2,
            value: 20.0,
            category: "B".to_string(),
            valid: true,
        });
        
        processor.records.push(DataRecord {
            id: 3,
            value: 30.0,
            category: "A".to_string(),
            valid: false,
        });
        
        let average = processor.calculate_average();
        assert_eq!(average, Some(15.0));
    }

    #[test]
    fn test_group_by_category() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(DataRecord {
            id: 1,
            value: 10.5,
            category: "CategoryA".to_string(),
            valid: true,
        });
        
        processor.records.push(DataRecord {
            id: 2,
            value: 20.3,
            category: "CategoryB".to_string(),
            valid: false,
        });
        
        processor.records.push(DataRecord {
            id: 3,
            value: 15.7,
            category: "CategoryA".to_string(),
            valid: true,
        });
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("CategoryA").unwrap().len(), 2);
        assert_eq!(groups.get("CategoryB").unwrap().len(), 1);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].trim().to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[3].trim().to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.75,CategoryA").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(
            1,
            "Test1".to_string(),
            10.0,
            "CategoryA".to_string(),
        ));
        processor.records.push(DataRecord::new(
            2,
            "Test2".to_string(),
            20.0,
            "CategoryB".to_string(),
        ));
        processor.records.push(DataRecord::new(
            3,
            "Test3".to_string(),
            30.0,
            "CategoryA".to_string(),
        ));

        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(
            1,
            "Test1".to_string(),
            10.0,
            "A".to_string(),
        ));
        processor.records.push(DataRecord::new(
            2,
            "Test2".to_string(),
            20.0,
            "A".to_string(),
        ));
        processor.records.push(DataRecord::new(
            3,
            "Test3".to_string(),
            30.0,
            "A".to_string(),
        ));

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }
}