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

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }
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
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(Record::new(id, name, value, category));
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

pub fn find_max_value(records: &[&Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_totals: HashMap<String, f64> = HashMap::new();

    for record in records {
        *category_totals.entry(record.category.clone()).or_insert(0.0) += record.value;
    }

    let mut result: Vec<(String, f64)> = category_totals.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records() -> Vec<Record> {
        vec![
            Record::new(1, "ItemA".to_string(), 10.5, "CategoryX".to_string()),
            Record::new(2, "ItemB".to_string(), 20.3, "CategoryY".to_string()),
            Record::new(3, "ItemC".to_string(), 15.7, "CategoryX".to_string()),
            Record::new(4, "ItemD".to_string(), 8.9, "CategoryZ".to_string()),
            Record::new(5, "ItemE".to_string(), 12.1, "CategoryY".to_string()),
        ]
    }

    #[test]
    fn test_filter_by_category() {
        let records = create_test_records();
        let filtered = filter_by_category(&records, "CategoryX");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "CategoryX"));
    }

    #[test]
    fn test_calculate_average() {
        let records = create_test_records();
        let filtered = filter_by_category(&records, "CategoryX");
        let avg = calculate_average(&filtered).unwrap();
        assert!((avg - 13.1).abs() < 0.001);
    }

    #[test]
    fn test_find_max_value() {
        let records = create_test_records();
        let max_record = find_max_value(&records.iter().collect::<Vec<_>>()).unwrap();
        assert_eq!(max_record.id, 2);
        assert!((max_record.value - 20.3).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = create_test_records();
        let aggregates = aggregate_by_category(&records);
        
        let mut expected = vec![
            ("CategoryX".to_string(), 26.2),
            ("CategoryY".to_string(), 32.4),
            ("CategoryZ".to_string(), 8.9),
        ];
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        
        for (i, (category, total)) in aggregates.iter().enumerate() {
            assert_eq!(category, &expected[i].0);
            assert!((total - expected[i].1).abs() < 0.001);
        }
    }
}