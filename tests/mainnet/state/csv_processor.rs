
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid number of fields at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = parts[1].trim().to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = parts[3].trim().parse::<bool>()
            .map_err(|e| format!("Invalid active flag at line {}: {}", line_number, e))?;

        let record = Record::new(id, name, value, active);
        
        if let Err(e) = record.validate() {
            return Err(format!("Validation error at line {}: {}", line_number, e).into());
        }
        
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / records.len() as f64;
    let active_count = records.iter().filter(|r| r.active).count();

    (sum, avg, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,33.7,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,15.2,true").unwrap();

        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 33.7);
        assert_eq!(records[2].active, true);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_name = Record::new(2, "".to_string(), 5.0, false);
        assert!(invalid_name.validate().is_err());

        let invalid_value = Record::new(3, "Test".to_string(), -5.0, true);
        assert!(invalid_value.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, false),
            Record::new(3, "C".to_string(), 30.0, true),
        ];

        let (sum, avg, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert!(records.is_empty());

        let stats = calculate_statistics(&records);
        assert_eq!(stats, (0.0, 0.0, 0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_headers {
            let _ = lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .into_iter()
            .filter(|record| predicate(record))
            .collect()
    }

    pub fn transform_column<F>(
        &self,
        records: Vec<Vec<String>>,
        column_index: usize,
        transformer: F,
    ) -> Vec<Vec<String>>
    where
        F: Fn(&str) -> String,
    {
        records
            .into_iter()
            .map(|mut record| {
                if column_index < record.len() {
                    record[column_index] = transformer(&record[column_index]);
                }
                record
            })
            .collect()
    }
}

pub fn calculate_column_average(records: &[Vec<String>], column_index: usize) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0;

    for record in records {
        if column_index < record.len() {
            if let Ok(value) = record[column_index].parse::<f64>() {
                sum += value;
                count += 1;
            }
        }
    }

    if count > 0 {
        Some(sum / count as f64)
    } else {
        None
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
        writeln!(temp_file, "Alice,25,New York").unwrap();
        writeln!(temp_file, "Bob,30,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.read_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "25", "New York"]);

        let filtered = processor.filter_records(records, |record| {
            record.get(1).and_then(|age| age.parse::<i32>().ok()) > Some(30)
        });

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Paris"]);

        let transformed = processor.transform_column(filtered, 2, |city| city.to_uppercase());
        assert_eq!(transformed[0], vec!["Charlie", "35", "PARIS"]);
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];

        let avg = calculate_column_average(&records, 0);
        assert!(avg.is_some());
        assert!((avg.unwrap() - 12.666).abs() < 0.001);
    }
}