
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn save_filtered_to_csv<P: AsRef<Path>>(&self, path: P, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, id: u32, name: String, value: f64, category: String) {
        self.records.push(Record {
            id,
            name,
            value,
            category,
        });
    }

    fn sort_by_value(&mut self) {
        self.records.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(1, "Item A".to_string(), 42.5, "Alpha".to_string());
    processor.add_record(2, "Item B".to_string(), 37.8, "Beta".to_string());
    processor.add_record(3, "Item C".to_string(), 55.2, "Alpha".to_string());
    processor.add_record(4, "Item D".to_string(), 29.9, "Gamma".to_string());
    
    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    let alpha_records = processor.filter_by_category("Alpha");
    println!("Alpha category records: {}", alpha_records.len());
    
    processor.sort_by_value();
    println!("Sorted records:");
    for record in &processor.records {
        println!("  {}: {} - {:.1}", record.id, record.name, record.value);
    }
    
    Ok(())
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_data(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.is_empty() {
                errors.push(format!("Record {}: Name is empty", index));
            }
            if record.value < 0.0 {
                errors.push(format!("Record {}: Value is negative", index));
            }
            if record.category.is_empty() {
                errors.push(format!("Record {}: Category is empty", index));
            }
        }

        errors
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.0,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.75,Category1").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.416666666666666).abs() < 0.0001);

        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);

        let errors = processor.validate_data();
        assert!(errors.is_empty());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    pub file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
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
            return Err("No valid data found in CSV file".into());
        }

        Ok(records)
    }

    pub fn validate_numeric_fields(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();

        for (row_num, record) in records.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds at row {}", column_index, row_num + 1).into());
            }

            match record[column_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at row {}: {}", row_num + 1, record[column_index]).into()),
            }
        }

        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = if count > 0.0 { sum / count } else { 0.0 };
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
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
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let records = processor.process_csv().unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["name", "age", "salary"]);
    }

    #[test]
    fn test_numeric_validation() {
        let records = vec![
            vec!["100.5".to_string(), "text".to_string()],
            vec!["200.0".to_string(), "more".to_string()],
        ];
        
        let processor = DataProcessor::new("dummy.csv");
        let numeric_values = processor.validate_numeric_fields(&records, 0).unwrap();
        
        assert_eq!(numeric_values, vec![100.5, 200.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let values = vec![10.0, 20.0, 30.0, 40.0];
        let processor = DataProcessor::new("dummy.csv");
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert_eq!(mean, 25.0);
        assert_eq!(variance, 125.0);
        assert!((std_dev - 11.1803398875).abs() < 1e-10);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_result) = lines.next() {
            let header_line = header_result?;
            let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();

            for line_result in lines {
                let line = line_result?;
                let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        if let Ok(num) = values[i].parse::<f64>() {
                            record.insert(header.clone(), num);
                        }
                    }
                    self.records.push(record);
                }
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, field: &str) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(field).copied())
            .collect();

        if values.is_empty() {
            return None;
        }

        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }

    pub fn filter_records(&self, field: &str, threshold: f64) -> Vec<HashMap<String, f64>> {
        self.records
            .iter()
            .filter(|record| {
                record.get(field).map_or(false, |&value| value > threshold)
            })
            .cloned()
            .collect()
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
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,score").unwrap();
        writeln!(temp_file, "1,10.5,0.8").unwrap();
        writeln!(temp_file, "2,15.2,0.9").unwrap();
        writeln!(temp_file, "3,8.7,0.6").unwrap();

        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);

        let stats = processor.calculate_statistics("value");
        assert!(stats.is_some());
        
        let (mean, _, std_dev) = stats.unwrap();
        assert!((mean - 11.466666).abs() < 0.001);
        assert!(std_dev > 0.0);

        let filtered = processor.filter_records("score", 0.7);
        assert_eq!(filtered.len(), 2);
    }
}