use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn read_and_filter(&self, column_index: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let record: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            
            if record.get(column_index).map_or(false, |value| value == filter_value) {
                filtered_records.push(record);
            }
        }

        Ok(filtered_records)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        Ok(count)
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
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,alice,30").unwrap();
        writeln!(temp_file, "2,bob,25").unwrap();
        writeln!(temp_file, "3,alice,35").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let filtered = processor.read_and_filter(1, "alice").unwrap();
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "1");
        assert_eq!(filtered[1][0], "3");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let record = Record {
            id: parts[0].parse()?,
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse().unwrap_or(false),
        };
        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.category == category && r.active)
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

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record {
                id: 1,
                category: "A".to_string(),
                value: 10.5,
                active: true,
            },
            Record {
                id: 2,
                category: "B".to_string(),
                value: 20.0,
                active: true,
            },
            Record {
                id: 3,
                category: "A".to_string(),
                value: 15.0,
                active: false,
            },
        ];

        let filtered = filter_by_category(&records, "A");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record {
                id: 1,
                category: "Test".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                category: "Test".to_string(),
                value: 20.0,
                active: true,
            },
            Record {
                id: 3,
                category: "Test".to_string(),
                value: 30.0,
                active: true,
            },
        ];

        let refs: Vec<&Record> = records.iter().collect();
        let avg = calculate_average(&refs).unwrap();
        assert_eq!(avg, 20.0);
    }
}