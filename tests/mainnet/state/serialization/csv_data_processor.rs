use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: &str, value: f64, active: bool) -> Self {
        Self {
            id,
            category: category.to_string(),
            value,
            active,
        }
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        Self { records: Vec::new() }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);

        for result in reader.deserialize() {
            let record: DataRecord = result?;
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

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    fn save_filtered_to_csv(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut writer = Writer::from_path(file_path)?;

        for record in filtered {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    fn remove_inactive(&mut self) {
        self.records.retain(|record| record.active);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();

    processor.add_record(DataRecord::new(1, "A", 100.5, true));
    processor.add_record(DataRecord::new(2, "B", 200.3, false));
    processor.add_record(DataRecord::new(3, "A", 150.7, true));
    processor.add_record(DataRecord::new(4, "C", 300.9, true));

    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());

    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }

    let category_a_records = processor.filter_by_category("A");
    println!("Category A records: {}", category_a_records.len());

    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());

    processor.remove_inactive();
    println!("Records after removing inactive: {}", processor.records.len());

    processor.save_filtered_to_csv("filtered_data.csv", "A")?;
    println!("Filtered data saved to filtered_data.csv");

    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let line = line?;
            let fields: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name);

        match column_index {
            Some(idx) => self.records.iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, String> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Ok(value) = record[column_index].parse::<f64>() {
                sum += value;
                count += 1;
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Err("No numeric values found in column".to_string())
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary").unwrap();
        writeln!(file, "Alice,30,50000").unwrap();
        writeln!(file, "Bob,25,45000").unwrap();
        writeln!(file, "Charlie,35,60000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), &vec!["name".to_string(), "age".to_string(), "salary".to_string()]);
        assert_eq!(processor.get_record_count(), 3);
    }

    #[test]
    fn test_filtering() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("age", |age| age.parse::<i32>().unwrap() > 30);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Charlie");
    }

    #[test]
    fn test_aggregation() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let avg_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert!((avg_salary - 51666.666).abs() < 0.001);
    }
}use csv::{Reader, Writer};
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

fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_records(records: &[Record], category_filter: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter && r.active)
        .cloned()
        .collect()
}

fn aggregate_values(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

fn save_results<P: AsRef<Path>>(records: &[Record], path: P) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = Writer::from_writer(file);
    
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_data(input_path: &str, output_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv(input_path)?;
    let filtered = filter_records(&all_records, category);
    
    if filtered.is_empty() {
        println!("No records found for category: {}", category);
        return Ok(());
    }
    
    let total_value = aggregate_values(&filtered);
    println!("Processed {} records", filtered.len());
    println!("Total value for category '{}': {:.2}", category, total_value);
    
    save_results(&filtered, output_path)?;
    println!("Results saved to: {}", output_path);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let target_category = "electronics";
    
    process_data(input_file, output_file, target_category)
}use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self { id, category, value, active }
    }
}

fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: DataRecord = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records.iter()
        .filter(|r| r.active)
        .cloned()
        .collect()
}

fn calculate_category_averages(records: &[DataRecord]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut category_sums: HashMap<String, (f64, usize)> = HashMap::new();
    
    for record in records {
        let entry = category_sums.entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }
    
    category_sums.into_iter()
        .map(|(category, (sum, count))| (category, sum / count as f64))
        .collect()
}

fn save_processed_data(file_path: &str, records: &[DataRecord]) -> Result<(), Box<dyn Error>> {
    let file = File::create(file_path)?;
    let mut writer = Writer::from_writer(file);
    
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_data_pipeline(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv_data(input_path)?;
    println!("Loaded {} total records", all_records.len());
    
    let active_records = filter_active_records(&all_records);
    println!("Found {} active records", active_records.len());
    
    let averages = calculate_category_averages(&active_records);
    println!("Category averages:");
    for (category, avg) in averages {
        println!("  {}: {:.2}", category, avg);
    }
    
    save_processed_data(output_path, &active_records)?;
    println!("Saved processed data to {}", output_path);
    
    Ok(())
}

fn main() {
    let input_file = "data/input.csv";
    let output_file = "data/processed.csv";
    
    match process_data_pipeline(input_file, output_file) {
        Ok(()) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
}