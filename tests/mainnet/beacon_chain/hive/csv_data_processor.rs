use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            category,
            value,
            active,
        }
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);

        for result in reader.deserialize() {
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

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_active_records(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_path)?;
        let mut writer = Writer::from_writer(file);

        for record in self.records.iter().filter(|r| r.active) {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, "A".to_string(), 10.5, true));
        processor.add_record(DataRecord::new(2, "B".to_string(), 20.0, false));
        processor.add_record(DataRecord::new(3, "A".to_string(), 30.5, true));

        assert_eq!(processor.get_record_count(), 3);
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 20.333333).abs() < 0.0001);
    }

    #[test]
    fn test_export_functionality() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "Test".to_string(), 100.0, true));
        
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();
        
        let result = processor.export_active_records(output_path);
        assert!(result.is_ok());
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".to_string());
    }
    Ok(())
}

fn transform_record(record: &mut Record) {
    record.name = record.name.to_uppercase();
    record.value = (record.value * 100.0).round() / 100.0;
}

fn process_csv_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_path)?;
    
    let mut writer = WriterBuilder::new()
        .from_path(output_path)?;
    
    writer.write_record(&["id", "name", "value", "category"])?;
    
    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        match validate_record(&record) {
            Ok(_) => {
                transform_record(&mut record);
                writer.serialize(&record)?;
            }
            Err(e) => eprintln!("Skipping invalid record: {}", e),
        }
    }
    
    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / count;
    let variance: f64 = records.iter()
        .map(|r| (r.value - avg).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();
    
    (sum, avg, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        assert!(validate_record(&valid_record).is_ok());
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "D".to_string(),
        };
        assert!(validate_record(&invalid_record).is_err());
    }
    
    #[test]
    fn test_transform_record() {
        let mut record = Record {
            id: 1,
            name: "test".to_string(),
            value: 10.12345,
            category: "A".to_string(),
        };
        
        transform_record(&mut record);
        assert_eq!(record.name, "TEST");
        assert_eq!(record.value, 10.12);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];
        
        let (sum, avg, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
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

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
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
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<&CsvRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|record| record.value).sum()
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "id,name,value,category\n1,ItemA,25.5,Electronics\n2,ItemB,42.8,Books\n3,ItemC,18.3,Electronics"
        )
        .unwrap();

        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);

        let electronics = filter_by_category(&records, "Electronics");
        assert_eq!(electronics.len(), 2);

        let total = calculate_total_value(&records);
        assert!((total - 86.6).abs() < 0.001);

        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.name, "ItemB");
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
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

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    fn filter_active(&self) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        let mut aggregates = std::collections::HashMap::new();
        
        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += record.value;
        }

        aggregates
            .into_iter()
            .map(|(category, (count, total))| (category, total / count as f64))
            .collect()
    }

    fn save_to_file<P: AsRef<Path>>(&self, path: P, records: &[Record]) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(path)?;
        for record in records {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    fn find_max_value_record(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn find_min_value_record(&self) -> Option<&Record> {
        self.records.iter().min_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

fn process_sample_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    let sample_data = "id,name,category,value,active
1,ItemA,Electronics,299.99,true
2,ItemB,Books,19.99,true
3,ItemC,Electronics,599.99,false
4,ItemD,Clothing,49.99,true
5,ItemE,Books,14.99,true";

    let temp_path = "temp_sample.csv";
    std::fs::write(temp_path, sample_data)?;
    
    processor.load_from_file(temp_path)?;
    
    println!("Total records: {}", processor.records.len());
    
    if let Some(avg) = processor.calculate_average_value() {
        println!("Average value: {:.2}", avg);
    }
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Electronics items: {}", electronics.len());
    
    let aggregates = processor.aggregate_by_category();
    for (category, avg) in aggregates {
        println!("Category {} average: {:.2}", category, avg);
    }
    
    if let Some(max_record) = processor.find_max_value_record() {
        println!("Max value record: {} - {}", max_record.name, max_record.value);
    }
    
    std::fs::remove_file(temp_path)?;
    Ok(())
}

fn main() {
    if let Err(e) = process_sample_data() {
        eprintln!("Error processing data: {}", e);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

impl CsvRecord {
    pub fn from_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err("Invalid CSV format".into());
        }

        Ok(CsvRecord {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
            active: parts[4].parse()?,
        })
    }
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

    pub fn load_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let record = CsvRecord::from_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn get_category_summary(&self) -> Vec<(String, f64, usize)> {
        use std::collections::HashMap;

        let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = category_map
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        category_map
            .into_iter()
            .map(|(category, (total, count))| (category, total, count))
            .collect()
    }

    pub fn find_max_value_record(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn find_min_value_record(&self) -> Option<&CsvRecord> {
        self.records.iter().min_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

pub fn process_csv_data(filepath: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file(filepath)?;

    println!("Total records loaded: {}", processor.records.len());
    println!("Total value: {:.2}", processor.calculate_total_value());
    println!("Average value: {:.2}", processor.calculate_average_value());

    if let Some(max_record) = processor.find_max_value_record() {
        println!("Max value record: {:?}", max_record);
    }

    if let Some(min_record) = processor.find_min_value_record() {
        println!("Min value record: {:?}", min_record);
    }

    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());

    let category_summary = processor.get_category_summary();
    for (category, total, count) in category_summary {
        println!("Category '{}': {} items, total value: {:.2}", category, count, total);
    }

    Ok(())
}