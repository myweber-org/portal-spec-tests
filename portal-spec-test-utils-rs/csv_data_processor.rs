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

fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: Vec<Record>) -> Vec<Record> {
    records.into_iter().filter(|r| r.active).collect()
}

fn calculate_category_averages(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_totals
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_totals
        .into_iter()
        .map(|(category, (total, count))| (category, total / count as f64))
        .collect()
}

fn save_results_to_csv(
    averages: &[(String, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    writer.write_record(&["Category", "AverageValue"])?;

    for (category, average) in averages {
        writer.write_record(&[category, &average.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    let active_records = filter_active_records(records);
    let category_averages = calculate_category_averages(&active_records);
    save_results_to_csv(&category_averages, output_path)?;

    println!("Processed {} active records", active_records.len());
    println!("Generated averages for {} categories", category_averages.len());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";

    match process_csv_data(input_file, output_file) {
        Ok(()) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err("Invalid CSV format".into());
        }

        Ok(DataRecord {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
            active: parts[4].parse()?,
        })
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            let record = DataRecord::from_csv_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn filter_active(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    fn process_data(&self) -> Vec<String> {
        let mut results = Vec::new();
        
        results.push(format!("Total records: {}", self.records.len()));
        results.push(format!("Average value: {:.2}", self.calculate_average_value()));
        
        if let Some(max_record) = self.find_max_value() {
            results.push(format!(
                "Max value: {} (ID: {}, Name: {})",
                max_record.value, max_record.id, max_record.name
            ));
        }
        
        let active_records = self.filter_active();
        results.push(format!("Active records: {}", active_records.len()));
        
        let categories = self.group_by_category();
        results.push(format!("Unique categories: {}", categories.len()));
        
        for (category, records) in categories {
            let category_avg: f64 = records.iter().map(|r| r.value).sum::<f64>() / records.len() as f64;
            results.push(format!(
                "Category '{}': {} records, average value: {:.2}",
                category,
                records.len(),
                category_avg
            ));
        }
        
        results
    }
}

fn generate_sample_data() -> Result<(), Box<dyn Error>> {
    let sample_data = vec![
        "1,Product A,Electronics,299.99,true",
        "2,Product B,Books,19.99,true",
        "3,Product C,Electronics,599.99,false",
        "4,Product D,Clothing,49.99,true",
        "5,Product E,Books,29.99,true",
        "6,Product F,Electronics,399.99,true",
        "7,Product G,Clothing,79.99,false",
        "8,Product H,Books,14.99,true",
    ];

    let mut file = File::create("sample_data.csv")?;
    writeln!(&mut file, "id,name,category,value,active")?;
    
    for line in sample_data {
        writeln!(&mut file, "{}", line)?;
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if let Err(_) = generate_sample_data() {
        eprintln!("Warning: Could not generate sample data file");
    }

    let mut processor = DataProcessor::new();
    
    match processor.load_from_file("sample_data.csv") {
        Ok(_) => {
            println!("Data loaded successfully");
            
            let results = processor.process_data();
            for result in results {
                println!("{}", result);
            }
            
            let electronics = processor.filter_by_category("Electronics");
            println!("\nElectronics products ({}):", electronics.len());
            for product in electronics {
                println!("  - {}: ${:.2}", product.name, product.value);
            }
        }
        Err(e) => {
            eprintln!("Error loading data: {}", e);
            eprintln!("Using empty dataset for demonstration");
            
            let results = processor.process_data();
            for result in results {
                println!("{}", result);
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::from_csv_line("1,Test,Category,100.5,true").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test");
        assert_eq!(record.category, "Category");
        assert_eq!(record.value, 100.5);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_invalid_csv() {
        let result = DataRecord::from_csv_line("1,Test,Category");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.records.len(), 0);
        assert_eq!(processor.calculate_average_value(), 0.0);
        assert!(processor.find_max_value().is_none());
    }
}