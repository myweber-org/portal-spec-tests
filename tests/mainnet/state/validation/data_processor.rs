
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            category,
        }
    }
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn save_filtered_results(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered_records = self.filter_by_category(category);
        
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in filtered_records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.records
            .iter()
            .map(|record| record.value)
            .fold(f64::NEG_INFINITY, f64::max);

        (count, avg, max)
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Category1").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        assert!((stats.1 - 15.5).abs() < 0.1);
        assert!((stats.2 - 20.3).abs() < 0.1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
    timestamp: String,
}

#[derive(Debug)]
struct DataProcessor {
    records: Vec<Record>,
    filtered_records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            filtered_records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        println!("Loaded {} records from {}", self.records.len(), file_path);
        Ok(())
    }

    fn filter_by_category(&mut self, category: &str) {
        self.filtered_records = self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect();

        println!("Filtered {} records in category '{}'", 
                 self.filtered_records.len(), category);
    }

    fn calculate_average_value(&self) -> Option<f64> {
        if self.filtered_records.is_empty() {
            return None;
        }

        let sum: f64 = self.filtered_records
            .iter()
            .map(|record| record.value)
            .sum();
        
        Some(sum / self.filtered_records.len() as f64)
    }

    fn save_filtered_to_csv(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in &self.filtered_records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        println!("Saved {} filtered records to {}", 
                 self.filtered_records.len(), output_path);
        Ok(())
    }

    fn find_max_value_record(&self) -> Option<&Record> {
        self.filtered_records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    processor.filter_by_category("premium");
    
    if let Some(average) = processor.calculate_average_value() {
        println!("Average value: {:.2}", average);
    }
    
    if let Some(max_record) = processor.find_max_value_record() {
        println!("Maximum value record: {:?}", max_record);
    }
    
    processor.save_filtered_to_csv("filtered_output.csv")?;
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
}