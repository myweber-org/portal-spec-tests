use std::error::Error;
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
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() == 4 {
            let id = parts[0].parse()?;
            let category = parts[1].to_string();
            let value = parts[2].parse()?;
            let active = parts[3].parse()?;
            
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

pub fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|record| record.active)
        .collect()
}

pub fn calculate_category_averages(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();
    
    for record in records {
        let entry = category_totals
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        
        entry.0 += record.value;
        entry.1 += 1;
    }
    
    category_totals
        .into_iter()
        .map(|(category, (total, count))| (category, total / count as f64))
        .collect()
}

pub fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter()
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
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
        writeln!(temp_file, "2,books,45.0,false").unwrap();
        
        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].category, "electronics");
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
    }

    #[test]
    fn test_calculate_category_averages() {
        let records = vec![
            Record { id: 1, category: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, category: "A".to_string(), value: 20.0, active: true },
            Record { id: 3, category: "B".to_string(), value: 30.0, active: true },
        ];
        
        let averages = calculate_category_averages(&records);
        assert_eq!(averages.len(), 2);
        
        let a_avg = averages.iter().find(|(cat, _)| cat == "A").unwrap();
        assert_eq!(a_avg.1, 15.0);
    }
}