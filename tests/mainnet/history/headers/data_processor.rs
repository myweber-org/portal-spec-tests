
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
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
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();

            if value < 0.0 {
                continue;
            }

            self.records.push(DataRecord {
                id,
                value,
                category,
            });
            count += 1;
        }

        Ok(count)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
        assert_eq!(processor.record_count(), 0);
        assert_eq!(processor.calculate_average(), None);
    }

    #[test]
    fn test_load_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,value,category").unwrap();
        writeln!(file, "1,10.5,type_a").unwrap();
        writeln!(file, "2,20.3,type_b").unwrap();
        writeln!(file, "3,15.7,type_a").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.record_count(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.1);
    }

    #[test]
    fn test_filter_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "A".to_string(),
        });
        processor.records.push(DataRecord {
            id: 2,
            value: 20.0,
            category: "B".to_string(),
        });
        processor.records.push(DataRecord {
            id: 3,
            value: 30.0,
            category: "A".to_string(),
        });

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_max_value() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "A".to_string(),
        });
        processor.records.push(DataRecord {
            id: 2,
            value: 30.0,
            category: "B".to_string(),
        });
        processor.records.push(DataRecord {
            id: 3,
            value: 20.0,
            category: "A".to_string(),
        });

        let max_record = processor.get_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
        assert_eq!(max_record.unwrap().value, 30.0);
    }
}