use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn filter_records(records: &[Record], category_filter: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter && r.active)
        .collect()
}

fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

fn process_csv_file(input_path: &str, output_path: &str, target_category: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    
    let mut all_records = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        all_records.push(record);
    }
    
    let filtered = filter_records(&all_records, target_category);
    let avg_value = calculate_average(&filtered);
    
    let output_file = File::create(output_path)?;
    let mut wtr = Writer::from_writer(output_file);
    
    for record in filtered {
        wtr.serialize(record)?;
    }
    
    wtr.flush()?;
    
    if let Some(avg) = avg_value {
        println!("Processed {} records with category '{}'", filtered.len(), target_category);
        println!("Average value: {:.2}", avg);
    } else {
        println!("No records found for category '{}'", target_category);
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let target_category = "electronics";
    
    process_csv_file(input_file, output_file, target_category)
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut line_count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
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
            
            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[3].to_string();
            
            self.records.push(CsvRecord {
                id,
                name,
                value,
                category,
            });
            
            line_count += 1;
        }
        
        Ok(line_count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value_record(&self) -> Option<&CsvRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn export_summary(&self) -> String {
        let avg = self.calculate_average_value().unwrap_or(0.0);
        let max_record = self.find_max_value_record();
        
        let max_info = match max_record {
            Some(record) => format!("ID: {}, Value: {}", record.id, record.value),
            None => "No records".to_string(),
        };
        
        format!(
            "Total records: {}\nAverage value: {:.2}\nMaximum value record: {}",
            self.records.len(),
            avg,
            max_info
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut processor = CsvProcessor::new();
        
        let csv_data = "id,name,value,category\n\
                       1,ItemA,10.5,Electronics\n\
                       2,ItemB,25.0,Books\n\
                       3,ItemC,15.75,Electronics\n\
                       4,ItemD,8.99,Books";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let count = processor.load_from_file(temp_file.path()).unwrap();
        assert_eq!(count, 4);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average_value().unwrap();
        assert!((avg - 15.06).abs() < 0.01);
        
        let max_record = processor.find_max_value_record().unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 25.0);
        
        processor.transform_values(|x| x * 1.1);
        let new_avg = processor.calculate_average_value().unwrap();
        assert!((new_avg - 16.566).abs() < 0.01);
    }
}