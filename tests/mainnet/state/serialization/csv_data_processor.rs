use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
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

            let record = CsvRecord::new(id, name, value, category);
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_all(&self) -> Vec<(usize, String)> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if let Err(err) = record.validate() {
                errors.push((index, err));
            }
        }

        errors
    }

    pub fn apply_transformation(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform_value(multiplier);
        }
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
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord::new(2, "".to_string(), -50.0, "".to_string());
        assert!(invalid_record.validate().is_err());
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
        processor.load_from_file(temp_file.path().to_str().unwrap())?;

        assert_eq!(processor.get_records().len(), 3);
        assert_eq!(processor.calculate_total_value(), 600.0);

        let category_a_items = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_items.len(), 2);

        processor.apply_transformation(2.0);
        assert_eq!(processor.calculate_total_value(), 1200.0);

        Ok(())
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
        })
    }
}

fn load_csv_records(filepath: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for line in reader.lines().skip(1) {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(Record::from_csv_line(&line)?);
    }

    Ok(records)
}

fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

fn calculate_average_value(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = load_csv_records("data.csv")?;
    
    println!("Total records loaded: {}", records.len());
    
    let filtered = filter_by_category(&records, "electronics");
    println!("Electronics records: {}", filtered.len());
    
    let avg = calculate_average_value(&records);
    println!("Average value: {:.2}", avg);
    
    if let Some(max_record) = find_max_value_record(&records) {
        println!("Highest value record: {} - {}", max_record.name, max_record.value);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_parsing() {
        let record = Record::from_csv_line("1,ProductA,electronics,99.99").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "ProductA");
        assert_eq!(record.category, "electronics");
        assert_eq!(record.value, 99.99);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), category: "electronics".to_string(), value: 10.0 },
            Record { id: 2, name: "B".to_string(), category: "books".to_string(), value: 20.0 },
            Record { id: 3, name: "C".to_string(), category: "electronics".to_string(), value: 30.0 },
        ];
        
        let filtered = filter_by_category(&records, "electronics");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), category: "test".to_string(), value: 10.0 },
            Record { id: 2, name: "B".to_string(), category: "test".to_string(), value: 20.0 },
            Record { id: 3, name: "C".to_string(), category: "test".to_string(), value: 30.0 },
        ];
        
        let avg = calculate_average_value(&records);
        assert_eq!(avg, 20.0);
    }
}