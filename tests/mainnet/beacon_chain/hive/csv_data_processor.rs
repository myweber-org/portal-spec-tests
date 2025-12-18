
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_columns: usize) -> bool {
        record.len() == expected_columns && record.iter().all(|field| !field.is_empty())
    }

    pub fn transform_numeric_fields(records: &mut Vec<Vec<String>>, column_index: usize) -> Result<(), Box<dyn Error>> {
        for record in records {
            if column_index < record.len() {
                let value = &record[column_index];
                if let Ok(num) = value.parse::<f64>() {
                    let transformed = (num * 100.0).round() / 100.0;
                    record[column_index] = format!("{:.2}", transformed);
                }
            }
        }
        Ok(())
    }

    pub fn filter_records<F>(records: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records.into_iter().filter(|record| predicate(record)).collect()
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "John,30,50000.5").unwrap();
        writeln!(temp_file, "Jane,25,60000.75").unwrap();
        writeln!(temp_file, "Bob,35,45000.0").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert!(processor.validate_record(&records[0], 3));
        
        let avg_age = calculate_column_average(&records, 1);
        assert!(avg_age.is_some());
        assert!((avg_age.unwrap() - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_numeric_transformation() {
        let mut records = vec![
            vec!["Alice".to_string(), "1234.567".to_string()],
            vec!["Bob".to_string(), "89.123".to_string()],
        ];
        
        CsvProcessor::transform_numeric_fields(&mut records, 1).unwrap();
        
        assert_eq!(records[0][1], "1234.57");
        assert_eq!(records[1][1], "89.12");
    }

    #[test]
    fn test_record_filtering() {
        let records = vec![
            vec!["active".to_string(), "user1".to_string()],
            vec!["inactive".to_string(), "user2".to_string()],
            vec!["active".to_string(), "user3".to_string()],
        ];
        
        let filtered = CsvProcessor::filter_records(records, |record| record[0] == "active");
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][1], "user1");
        assert_eq!(filtered[1][1], "user3");
    }
}