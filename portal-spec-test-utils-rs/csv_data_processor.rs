use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    category: parts[1].to_string(),
                    value: parts[2].parse()?,
                    active: parts[3].parse().unwrap_or(false),
                };
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
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

    pub fn aggregate_by_category(&self) -> HashMap<String, f64> {
        let mut aggregates = HashMap::new();

        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value;
        }

        aggregates
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "id,category,value,active\n1,electronics,250.5,true\n2,furniture,150.0,false\n3,electronics,75.25,true\n4,clothing,45.99,true"
        )
        .unwrap();
        temp_file
    }

    #[test]
    fn test_load_and_filter() {
        let temp_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        
        let count = processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 4);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 3);
    }

    #[test]
    fn test_aggregation() {
        let temp_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.get("electronics"), Some(&325.75));
        
        let average = processor.calculate_average();
        assert!((average - 130.435).abs() < 0.001);
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

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
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
                let record = CsvRecord {
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

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
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

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn find_min_value(&self) -> Option<&CsvRecord> {
        self.records.iter().min_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<CsvRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_all_records(&self) -> &Vec<CsvRecord> {
        &self.records
    }
}

pub fn process_csv_data(file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file(file_path)?;
    
    println!("Total records: {}", processor.count_records());
    println!("Total value: {:.2}", processor.calculate_total_value());
    println!("Average value: {:.2}", processor.calculate_average_value());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: ID {}, Name: {}, Value: {:.2}", 
                 max_record.id, max_record.name, max_record.value);
    }
    
    if let Some(min_record) = processor.find_min_value() {
        println!("Min value record: ID {}, Name: {}, Value: {:.2}", 
                 min_record.id, min_record.name, min_record.value);
    }
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    let groups = processor.group_by_category();
    for (category, records) in groups {
        let total: f64 = records.iter().map(|r| r.value).sum();
        println!("Category '{}': {} records, total value: {:.2}", 
                 category, records.len(), total);
    }
    
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub fn read_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line_num == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let category = parts[3].to_string();

        let record = CsvRecord::new(id, name, value, category);
        if let Err(e) = record.validate() {
            return Err(format!("Validation error at line {}: {}", line_num + 1, e).into());
        }

        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<CsvRecord> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn write_filtered_results(
    records: &[CsvRecord],
    output_path: &str,
    category: &str,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(output_path)?;
    writeln!(file, "id,name,value,category")?;

    let filtered = filter_by_category(records, category);
    for record in filtered {
        writeln!(
            file,
            "{},{},{},{}",
            record.id, record.name, record.value, record.category
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord::new(2, "".to_string(), -50.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,100.0,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();

        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Item1");
        assert_eq!(records[1].value, 200.0);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            CsvRecord::new(1, "A1".to_string(), 10.0, "X".to_string()),
            CsvRecord::new(2, "B1".to_string(), 20.0, "Y".to_string()),
            CsvRecord::new(3, "A2".to_string(), 30.0, "X".to_string()),
        ];

        let filtered = filter_by_category(&records, "X");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord::new(1, "Item1".to_string(), 25.5, "A".to_string()),
            CsvRecord::new(2, "Item2".to_string(), 34.5, "B".to_string()),
        ];

        let total = calculate_total_value(&records);
        assert_eq!(total, 60.0);
    }
}