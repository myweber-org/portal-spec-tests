use std::error::Error;
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
    records.iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_total_value(records: &[Record]) -> f64 {
    records.iter()
        .map(|record| record.value)
        .sum()
}

pub fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter()
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records() -> Vec<Record> {
        vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                value: 100.5,
                category: "Electronics".to_string(),
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                value: 75.0,
                category: "Books".to_string(),
            },
            Record {
                id: 3,
                name: "Item C".to_string(),
                value: 150.25,
                category: "Electronics".to_string(),
            },
        ]
    }

    #[test]
    fn test_filter_by_category() {
        let records = create_test_records();
        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = create_test_records();
        let total = calculate_total_value(&records);
        assert_eq!(total, 325.75);
    }

    #[test]
    fn test_find_max_value_record() {
        let records = create_test_records();
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 3);
        assert_eq!(max_record.value, 150.25);
    }
}