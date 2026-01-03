use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);
        
        for record in &self.records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = if count > 0.0 { sum / count } else { 0.0 };
        
        let variance: f64 = if count > 0.0 {
            values.iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count
        } else {
            0.0
        };
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
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

fn process_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(1, "Item A".to_string(), 45.6, "Category1".to_string());
    processor.add_record(2, "Item B".to_string(), 32.1, "Category2".to_string());
    processor.add_record(3, "Item C".to_string(), 67.8, "Category1".to_string());
    processor.add_record(4, "Item D".to_string(), 23.4, "Category3".to_string());
    processor.add_record(5, "Item E".to_string(), 89.0, "Category2".to_string());
    
    processor.sort_by_value();
    
    let (mean, variance, std_dev) = processor.calculate_statistics();
    println!("Statistics - Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}", 
             mean, variance, std_dev);
    
    let category1_items = processor.filter_by_category("Category1");
    println!("Category1 items: {}", category1_items.len());
    
    processor.save_to_csv("output_data.csv")?;
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data() {
        eprintln!("Error processing data: {}", e);
    }
}