use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

#[derive(Debug)]
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
}

fn read_csv_data(file_path: &Path) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn aggregate_by_category(records: &[Record]) -> Vec<AggregatedData> {
    use std::collections::HashMap;

    let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_map.entry(record.category.clone()).or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_map
        .into_iter()
        .map(|(category, (total, count))| AggregatedData {
            category,
            total_value: total,
            average_value: total / count as f64,
            record_count: count,
        })
        .collect()
}

fn write_aggregated_data(
    aggregated: &[AggregatedData],
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    for data in aggregated {
        writer.serialize(data)?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(input_path);
    let output_path = Path::new(output_path);

    let records = read_csv_data(input_path)?;
    println!("Total records read: {}", records.len());

    let active_records = filter_active_records(&records);
    println!("Active records: {}", active_records.len());

    let aggregated_data = aggregate_by_category(&records);
    println!("Categories aggregated: {}", aggregated_data.len());

    for data in &aggregated_data {
        println!(
            "Category: {}, Total: {:.2}, Average: {:.2}, Count: {}",
            data.category, data.total_value, data.average_value, data.record_count
        );
    }

    write_aggregated_data(&aggregated_data, output_path)?;
    println!("Results written to: {}", output_path.display());

    Ok(())
}

fn main() {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";

    if let Err(e) = process_csv_data(input_file, output_file) {
        eprintln!("Error processing CSV data: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 100.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Books".to_string(),
                value: 50.0,
                active: false,
            },
        ];

        let active = filter_active_records(&records);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 100.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Electronics".to_string(),
                value: 200.0,
                active: true,
            },
        ];

        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].total_value, 300.0);
        assert_eq!(aggregated[0].average_value, 150.0);
        assert_eq!(aggregated[0].record_count, 2);
    }

    #[test]
    fn test_csv_roundtrip() -> Result<(), Box<dyn Error>> {
        let temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;

        let test_data = "id,name,category,value,active\n1,Test Item,Test Category,42.5,true\n";
        std::fs::write(temp_input.path(), test_data)?;

        process_csv_data(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
        )?;

        assert!(temp_output.path().exists());
        Ok(())
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let record = Record {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    category: parts[2].to_string(),
                    value: parts[3].parse()?,
                    active: parts[4].parse().unwrap_or(false),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
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

    pub fn find_max_value_record(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn find_min_value_record(&self) -> Option<&Record> {
        self.records.iter().min_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_all_records(&self) -> &Vec<Record> {
        &self.records
    }
}

pub fn process_csv_data(file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file(file_path)?;
    
    println!("Total records loaded: {}", processor.count_records());
    println!("Total value: {:.2}", processor.calculate_total_value());
    println!("Average value: {:.2}", processor.calculate_average_value());
    
    if let Some(max_record) = processor.find_max_value_record() {
        println!("Maximum value record: {:?}", max_record);
    }
    
    if let Some(min_record) = processor.find_min_value_record() {
        println!("Minimum value record: {:?}", min_record);
    }
    
    let summary = processor.get_category_summary();
    println!("Category summary:");
    for (category, total, count) in summary {
        println!("  {}: {} items, total value: {:.2}", category, count, total);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor() {
        let csv_data = "id,name,category,value,active\n\
                        1,ItemA,Electronics,100.50,true\n\
                        2,ItemB,Furniture,75.25,false\n\
                        3,ItemC,Electronics,150.75,true\n\
                        4,ItemD,Furniture,50.00,true";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 4);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 3);
        
        let total_value = processor.calculate_total_value();
        assert!((total_value - 376.50).abs() < 0.01);
        
        let avg_value = processor.calculate_average_value();
        assert!((avg_value - 94.125).abs() < 0.01);
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

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
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
                
                let record = CsvRecord::new(id, name, value, category);
                if record.is_valid() {
                    self.records.push(record);
                }
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn apply_transformation(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform_value(multiplier);
        }
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
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
    fn test_csv_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = CsvRecord::new(2, "".to_string(), -50.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,value,category")?;
        writeln!(temp_file, "1,Item1,100.0,CategoryA")?;
        writeln!(temp_file, "2,Item2,200.0,CategoryB")?;
        writeln!(temp_file, "3,Item3,300.0,CategoryA")?;

        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path())?;

        assert_eq!(processor.get_records().len(), 3);
        assert_eq!(processor.calculate_total_value(), 600.0);
        
        let category_a_records = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_records.len(), 2);

        processor.apply_transformation(2.0);
        assert_eq!(processor.calculate_total_value(), 1200.0);

        Ok(())
    }
}