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
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].trim().to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].trim().to_string();
            
            if !Self::validate_record(&name, value, &category) {
                return Err(format!("Validation failed at line {}", line_num + 1).into());
            }
            
            self.records.push(CsvRecord {
                id,
                name,
                value,
                category,
            });
            
            count += 1;
        }
        
        Ok(count)
    }
    
    fn validate_record(name: &str, value: f64, category: &str) -> bool {
        !name.is_empty() && 
        value >= 0.0 && 
        value <= 10000.0 && 
        !category.is_empty()
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }
    
    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }
    
    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.calculate_total_value() / self.records.len() as f64)
        }
    }
    
    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
    }
    
    pub fn sort_by_value(&mut self) {
        self.records.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }
    
    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
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
        writeln!(temp_file, "1,Item A,100.5,Electronics").unwrap();
        writeln!(temp_file, "2,Item B,250.0,Books").unwrap();
        writeln!(temp_file, "3,Item C,75.25,Electronics").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let total = processor.calculate_total_value();
        assert!((total - 425.75).abs() < 0.001);
        
        let avg = processor.get_average_value();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 141.916).abs() < 0.001);
    }
    
    #[test]
    fn test_value_transformation() {
        let mut processor = CsvProcessor::new();
        processor.records.push(CsvRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        });
        
        processor.transform_values(|x| x * 1.1);
        assert!((processor.records[0].value - 110.0).abs() < 0.001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn read_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line_number == 1 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let category = parts[3].to_string();

        records.push(CsvRecord {
            id,
            name,
            value,
            category,
        });
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<&CsvRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average_value(records: &[CsvRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn aggregate_by_category(records: &[CsvRecord]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut category_totals: HashMap<String, f64> = HashMap::new();
    
    for record in records {
        *category_totals.entry(record.category.clone()).or_insert(0.0) += record.value;
    }
    
    let mut result: Vec<(String, f64)> = category_totals.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        
        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.3);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let filtered = filter_by_category(&records, "Cat1");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_average_value() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let avg = calculate_average_value(&records);
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_find_max_value_record() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            CsvRecord { id: 2, name: "B".to_string(), value: 30.0, category: "Cat2".to_string() },
            CsvRecord { id: 3, name: "C".to_string(), value: 20.0, category: "Cat1".to_string() },
        ];
        
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 30.0);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 2);
        assert_eq!(aggregated[0], ("Cat1".to_string(), 40.0));
        assert_eq!(aggregated[1], ("Cat2".to_string(), 20.0));
    }
}