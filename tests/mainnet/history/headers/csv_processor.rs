use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            headers: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_line) = lines.next() {
            let header_line = header_line?;
            self.headers = header_line.split(',').map(|s| s.trim().to_string()).collect();
        }

        for line in lines {
            let line = line?;
            let record: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if record.len() == self.headers.len() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(index) => index,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }

    pub fn get_column_summary(&self, column_name: &str) -> Option<(usize, String)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let values: Vec<&String> = self.records
            .iter()
            .filter_map(|record| record.get(column_index))
            .collect();

        if values.is_empty() {
            return None;
        }

        let unique_count = values.iter().collect::<std::collections::HashSet<_>>().len();
        let sample_value = values[0].clone();

        Some((unique_count, sample_value))
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn header_count(&self) -> usize {
        self.headers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,Alice,30").unwrap();
        writeln!(temp_file, "2,Bob,25").unwrap();
        writeln!(temp_file, "3,Alice,35").unwrap();
        temp_file
    }

    #[test]
    fn test_load_and_filter() {
        let temp_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.header_count(), 3);
        
        let alice_records = processor.filter_by_column("name", "Alice");
        assert_eq!(alice_records.len(), 2);
        
        let summary = processor.get_column_summary("name").unwrap();
        assert_eq!(summary.0, 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_row(row: &csv::StringRecord) -> Result<Self, Box<dyn Error>> {
        Ok(Record {
            id: row.get(0).ok_or("Missing ID")?.parse()?,
            name: row.get(1).ok_or("Missing name")?.to_string(),
            category: row.get(2).ok_or("Missing category")?.to_string(),
            value: row.get(3).ok_or("Missing value")?.parse()?,
            active: row.get(4).ok_or("Missing active flag")?.parse()?,
        })
    }

    fn to_row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.name.clone(),
            self.category.clone(),
            self.value.to_string(),
            self.active.to_string(),
        ]
    }

    fn filter_by_category(&self, target_category: &str) -> bool {
        self.category == target_category
    }

    fn apply_discount(&mut self, discount_percentage: f64) {
        if discount_percentage > 0.0 && discount_percentage <= 100.0 {
            self.value *= (100.0 - discount_percentage) / 100.0;
        }
    }
}

fn process_csv(
    input_path: &str,
    output_path: &str,
    filter_category: Option<&str>,
    discount: Option<f64>,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().from_writer(writer);

    let headers = vec!["ID", "Name", "Category", "Value", "Active"];
    csv_writer.write_record(&headers)?;

    for result in csv_reader.records() {
        let row = result?;
        let mut record = Record::from_row(&row)?;

        if let Some(category) = filter_category {
            if !record.filter_by_category(category) {
                continue;
            }
        }

        if let Some(discount_value) = discount {
            record.apply_discount(discount_value);
        }

        csv_writer.write_record(&record.to_row())?;
    }

    csv_writer.flush()?;
    Ok(())
}

fn validate_discount(discount: f64) -> Result<(), String> {
    if discount < 0.0 || discount > 100.0 {
        Err(format!("Discount must be between 0 and 100, got {}", discount))
    } else {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let target_category = Some("Electronics");
    let discount_amount = Some(15.0);

    if let Some(discount) = discount_amount {
        validate_discount(discount)?;
    }

    println!("Processing CSV data...");
    process_csv(input_file, output_file, target_category, discount_amount)?;
    println!("CSV processing completed successfully.");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_record_creation() {
        let row = csv::StringRecord::from(vec!["1", "Laptop", "Electronics", "999.99", "true"]);
        let record = Record::from_row(&row).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Laptop");
        assert_eq!(record.category, "Electronics");
        assert_eq!(record.value, 999.99);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_filter_by_category() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            category: "Books".to_string(),
            value: 10.0,
            active: true,
        };
        assert!(record.filter_by_category("Books"));
        assert!(!record.filter_by_category("Electronics"));
    }

    #[test]
    fn test_apply_discount() {
        let mut record = Record {
            id: 1,
            name: "Test".to_string(),
            category: "Test".to_string(),
            value: 100.0,
            active: true,
        };
        record.apply_discount(20.0);
        assert_eq!(record.value, 80.0);
    }

    #[test]
    fn test_validate_discount() {
        assert!(validate_discount(0.0).is_ok());
        assert!(validate_discount(50.0).is_ok());
        assert!(validate_discount(100.0).is_ok());
        assert!(validate_discount(-10.0).is_err());
        assert!(validate_discount(150.0).is_err());
    }
}