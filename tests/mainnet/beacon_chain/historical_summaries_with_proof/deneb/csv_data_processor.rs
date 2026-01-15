use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line.split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
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
    
    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
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
    
    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records.iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .collect();
        
        if numeric_values.is_empty() {
            return None;
        }
        
        match operation.to_lowercase().as_str() {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }
    
    pub fn write_to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        
        writeln!(file, "{}", self.headers.join(","))?;
        
        for record in &self.records {
            writeln!(file, "{}", record.join(","))?;
        }
        
        Ok(())
    }
    
    pub fn group_by_column(&self, column_name: &str) -> Option<HashMap<String, Vec<Vec<String>>>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)?;
        
        let mut groups: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        
        for record in &self.records {
            let key = record[column_index].clone();
            groups.entry(key)
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        Some(groups)
    }
}

pub fn process_csv_data(input_path: &str, output_path: &str, filter_column: &str, filter_value: &str) -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::from_file(input_path)?;
    
    let filtered_data = processor.filter_by_column(filter_column, |value| value == filter_value);
    
    let mut output_processor = CsvProcessor {
        headers: processor.headers.clone(),
        records: filtered_data,
    };
    
    output_processor.write_to_file(output_path)?;
    
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() == 4 {
            let record = Record {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                category: parts[3].to_string(),
            };
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[&Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,15.2,Category2").unwrap();

        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 15.2);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];

        let filtered = filter_by_category(&records, "Cat1");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "Cat1"));
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];

        let filtered = filter_by_category(&records, "Cat1");
        let avg = calculate_average(&filtered);
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 25.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 15.0, category: "Cat1".to_string() },
        ];

        let max_record = find_max_value(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 25.0);
    }
}use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;

#[derive(Debug, serde::Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_csv_file(file_path: &str, min_value: f64) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut filtered_records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= min_value && record.active {
            filtered_records.push(record);
        }
    }

    Ok(filtered_records)
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv", 100.0)?;
    
    println!("Filtered Records:");
    for record in records {
        println!("ID: {}, Name: {}, Value: {}", record.id, record.name, record.value);
    }
    
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
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
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = self.headers
            .iter()
            .position(|h| h == column_name);

        match column_index {
            Some(idx) => self.records
                .iter()
                .filter(|record| record.get(idx).map_or(false, |v| v == value))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn get_column_sum(&self, column_name: &str) -> Result<f64, Box<dyn Error>> {
        let column_index = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let mut sum = 0.0;
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                sum += value.parse::<f64>().unwrap_or(0.0);
            }
        }

        Ok(sum)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn headers(&self) -> &[String] {
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
        writeln!(file, "Charlie,30,60000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.headers(), vec!["name", "age", "salary"]);
        assert_eq!(processor.record_count(), 3);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("age", "30");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Charlie");
    }

    #[test]
    fn test_column_sum() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let sum = processor.get_column_sum("salary").unwrap();
        assert!((sum - 155000.0).abs() < 0.001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    name: String,
    category: String,
    value: f64,
    timestamp: String,
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
            timestamp: parts[4].to_string(),
        })
    }

    fn to_csv_line(&self) -> String {
        format!("{},{},{},{},{}", self.id, self.name, self.category, self.value, self.timestamp)
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

    fn load_from_file(&mut self, file_path: &Path) -> Result<(), Box<dyn Error>> {
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

    fn aggregate_by_category(&self) -> Vec<(String, f64, usize)> {
        use std::collections::HashMap;

        let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        aggregates
            .into_iter()
            .map(|(category, (total, count))| (category, total, count))
            .collect()
    }

    fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    fn save_filtered_results(&self, category: &str, output_path: &Path) -> io::Result<()> {
        let mut file = File::create(output_path)?;
        writeln!(file, "id,name,category,value,timestamp")?;

        for record in self.filter_by_category(category) {
            writeln!(file, "{}", record.to_csv_line())?;
        }

        Ok(())
    }

    fn generate_summary_report(&self) -> String {
        let total_records = self.records.len();
        let aggregates = self.aggregate_by_category();
        let max_record = self.find_max_value();

        let mut report = String::new();
        report.push_str(&format!("Total Records: {}\n", total_records));
        report.push_str("Category Aggregates:\n");

        for (category, total, count) in aggregates {
            report.push_str(&format!("  {}: {} items, total value: {:.2}\n", category, count, total));
        }

        if let Some(record) = max_record {
            report.push_str(&format!("Maximum Value Record: ID {} - {} ({}): {:.2}\n", 
                record.id, record.name, record.category, record.value));
        }

        report
    }
}

fn process_data_file(input_path: &str, output_category: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(input_path);
    let mut processor = DataProcessor::new();

    println!("Loading data from: {}", input_path);
    processor.load_from_file(path)?;

    println!("Generating summary report...");
    let report = processor.generate_summary_report();
    println!("{}", report);

    let output_path = format!("filtered_{}.csv", output_category);
    println!("Saving filtered results for '{}' to: {}", output_category, output_path);
    
    processor.save_filtered_results(output_category, Path::new(&output_path))?;

    Ok(())
}

fn main() {
    let input_file = "data.csv";
    
    if let Err(e) = process_data_file(input_file, "electronics") {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
    
    println!("Data processing completed successfully!");
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    
    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }
        
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let category = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let active = parts[3].parse::<bool>().unwrap_or(false);
            
            records.push(Record {
                id,
                category,
                value,
                active,
            });
        }
    }
    
    Ok(records)
}

pub fn filter_active_records(records: &[Record]) -> Vec<Record> {
    records.iter()
        .filter(|r| r.active)
        .cloned()
        .collect()
}

pub fn calculate_category_totals(records: &[Record]) -> HashMap<String, f64> {
    let mut totals = HashMap::new();
    
    for record in records {
        let entry = totals.entry(record.category.clone()).or_insert(0.0);
        *entry += record.value;
    }
    
    totals
}

pub fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter()
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn calculate_average_value(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,250.5,true").unwrap();
        writeln!(temp_file, "2,books,45.75,false").unwrap();
        
        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].category, "electronics");
        assert_eq!(records[1].value, 45.75);
    }
    
    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record { id: 1, category: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, category: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, category: "C".to_string(), value: 30.0, active: true },
        ];
        
        let active = filter_active_records(&records);
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|r| r.active));
    }
    
    #[test]
    fn test_calculate_category_totals() {
        let records = vec![
            Record { id: 1, category: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, category: "A".to_string(), value: 20.0, active: true },
            Record { id: 3, category: "B".to_string(), value: 30.0, active: true },
        ];
        
        let totals = calculate_category_totals(&records);
        assert_eq!(totals.get("A"), Some(&30.0));
        assert_eq!(totals.get("B"), Some(&30.0));
    }
}