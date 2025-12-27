
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
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
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_columns: usize) -> bool {
        record.len() == expected_columns && record.iter().all(|field| !field.is_empty())
    }

    pub fn transform_numeric_fields(records: &mut Vec<Vec<String>>, column_index: usize) -> Result<(), Box<dyn Error>> {
        for record in records {
            if column_index < record.len() {
                let value = &record[column_index];
                if let Ok(num) = value.parse::<f64>() {
                    let transformed = (num * 100.0).round() / 100.0;
                    record[column_index] = format!("{:.2}", transformed);
                }
            }
        }
        Ok(())
    }

    pub fn filter_records<F>(records: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records.into_iter().filter(|record| predicate(record)).collect()
    }
}

pub fn calculate_column_average(records: &[Vec<String>], column_index: usize) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0;

    for record in records {
        if column_index < record.len() {
            if let Ok(value) = record[column_index].parse::<f64>() {
                sum += value;
                count += 1;
            }
        }
    }

    if count > 0 {
        Some(sum / count as f64)
    } else {
        None
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "John,30,50000.5").unwrap();
        writeln!(temp_file, "Jane,25,60000.75").unwrap();
        writeln!(temp_file, "Bob,35,45000.0").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert!(processor.validate_record(&records[0], 3));
        
        let avg_age = calculate_column_average(&records, 1);
        assert!(avg_age.is_some());
        assert!((avg_age.unwrap() - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_numeric_transformation() {
        let mut records = vec![
            vec!["Alice".to_string(), "1234.567".to_string()],
            vec!["Bob".to_string(), "89.123".to_string()],
        ];
        
        CsvProcessor::transform_numeric_fields(&mut records, 1).unwrap();
        
        assert_eq!(records[0][1], "1234.57");
        assert_eq!(records[1][1], "89.12");
    }

    #[test]
    fn test_record_filtering() {
        let records = vec![
            vec!["active".to_string(), "user1".to_string()],
            vec!["inactive".to_string(), "user2".to_string()],
            vec!["active".to_string(), "user3".to_string()],
        ];
        
        let filtered = CsvProcessor::filter_records(records, |record| record[0] == "active");
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][1], "user1");
        assert_eq!(filtered[1][1], "user3");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let id = parts[0].parse::<u32>()?;
                let name = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let category = parts[3].to_string();

                self.records.push(Record::new(id, name, value, category));
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
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
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,25.5,CategoryX").unwrap();
        writeln!(temp_file, "2,ItemB,30.0,CategoryY").unwrap();
        writeln!(temp_file, "3,ItemC,42.8,CategoryX").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);

        let filtered = processor.filter_by_category("CategoryX");
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average();
        assert!((average - 32.77).abs() < 0.1);

        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 3);
    }
}