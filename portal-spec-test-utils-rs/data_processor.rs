use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    category: String,
    value: f64,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, (f64, f64, f64)> {
        let mut stats = HashMap::new();
        let mut category_data: HashMap<String, Vec<f64>> = HashMap::new();

        for record in &self.records {
            category_data
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.value);
        }

        for (category, values) in category_data {
            let count = values.len() as f64;
            let sum: f64 = values.iter().sum();
            let mean = sum / count;
            let variance: f64 = values.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>() / count;
            let std_dev = variance.sqrt();

            stats.insert(category, (mean, variance, std_dev));
        }

        stats
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn get_total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value").unwrap();
        writeln!(temp_file, "1,A,10.5").unwrap();
        writeln!(temp_file, "2,B,20.3").unwrap();
        writeln!(temp_file, "3,A,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_total_records(), 3);
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats.len(), 2);
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
    }
}
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

        for (line_number, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 3);
        assert!(processor.validate_record(&records[0]));

        let stats = processor.calculate_statistics(&records, 1);
        assert!(stats.is_some());

        let (mean, _, _) = stats.unwrap();
        assert!((mean - 30.0).abs() < 0.001);
    }
}