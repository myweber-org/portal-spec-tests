use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub fn parse_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line_content.split(',').collect();
        
        if fields.len() != 4 {
            return Err(format!("Invalid field count at line {}", line_number).into());
        }

        let id = fields[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = fields[1].trim().to_string();
        
        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = fields[3].trim().parse::<bool>()
            .map_err(|e| format!("Invalid active flag at line {}: {}", line_number, e))?;

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    if records.is_empty() {
        return Err("No valid records found in CSV file".into());
    }

    Ok(records)
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter()
        .filter(|r| r.active)
        .map(|r| r.value)
        .sum()
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter()
        .filter(|r| r.active)
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,12.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,8.75,false").unwrap();
        writeln!(temp_file, "3,ItemC,21.0,true").unwrap();
        
        let records = parse_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 8.75);
        assert!(!records[1].active);
    }

    #[test]
    fn test_calculate_total_active() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let total = calculate_total_value(&records);
        assert_eq!(total, 40.0);
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            CsvRecord { id: 1, name: "Low".to_string(), value: 5.0, active: true },
            CsvRecord { id: 2, name: "High".to_string(), value: 15.0, active: true },
            CsvRecord { id: 3, name: "Inactive".to_string(), value: 50.0, active: false },
        ];
        
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 15.0);
    }
}