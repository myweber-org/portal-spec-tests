use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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

pub fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,25.0,Books").unwrap();
        writeln!(temp_file, "3,ItemC,15.75,Electronics").unwrap();
        writeln!(temp_file, "4,ItemD,8.99,Books").unwrap();
        temp_file
    }

    #[test]
    fn test_load_csv() {
        let temp_file = create_test_csv();
        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(records.len(), 4);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 25.0);
    }

    #[test]
    fn test_filter_by_category() {
        let temp_file = create_test_csv();
        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        let electronics = filter_by_category(&records, "Electronics");
        
        assert_eq!(electronics.len(), 2);
        assert!(electronics.iter().all(|r| r.category == "Electronics"));
    }

    #[test]
    fn test_calculate_average() {
        let temp_file = create_test_csv();
        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        let electronics = filter_by_category(&records, "Electronics");
        let avg = calculate_average(&electronics).unwrap();
        
        assert!((avg - 13.125).abs() < 0.001);
    }

    #[test]
    fn test_find_max_value() {
        let temp_file = create_test_csv();
        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        let max_record = find_max_value(&records).unwrap();
        
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 25.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvProcessor {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl CsvProcessor {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
        let mut data = Vec::new();
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            data.push(row);
        }
        
        Ok(CsvProcessor { data, headers })
    }
    
    pub fn filter_by_column(&self, column_index: usize, predicate: fn(&str) -> bool) -> Vec<Vec<String>> {
        self.data
            .iter()
            .filter(|row| {
                if let Some(cell) = row.get(column_index) {
                    predicate(cell)
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    pub fn aggregate_numeric_column(&self, column_index: usize) -> Result<f64, String> {
        let mut sum = 0.0;
        let mut count = 0;
        
        for row in &self.data {
            if let Some(cell) = row.get(column_index) {
                match cell.parse::<f64>() {
                    Ok(value) => {
                        sum += value;
                        count += 1;
                    }
                    Err(_) => continue,
                }
            }
        }
        
        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Err("No numeric values found in column".to_string())
        }
    }
    
    pub fn get_unique_values(&self, column_index: usize) -> Vec<String> {
        let mut unique_values = std::collections::HashSet::new();
        
        for row in &self.data {
            if let Some(cell) = row.get(column_index) {
                unique_values.insert(cell.clone());
            }
        }
        
        unique_values.into_iter().collect()
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();
        
        let processor = CsvProcessor::new(temp_file.path()).unwrap();
        assert_eq!(processor.row_count(), 3);
        assert_eq!(processor.column_count(), 3);
        
        let filtered = processor.filter_by_column(1, |age| age.parse::<i32>().unwrap_or(0) > 30);
        assert_eq!(filtered.len(), 1);
        
        let avg_age = processor.aggregate_numeric_column(1).unwrap();
        assert!((avg_age - 30.0).abs() < 0.001);
        
        let unique_cities = processor.get_unique_values(2);
        assert_eq!(unique_cities.len(), 3);
    }
}