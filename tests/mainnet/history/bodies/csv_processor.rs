
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

pub fn parse_csv(file_path: &Path) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;

        let record = Record::new(id, name, value, active);
        if let Err(e) = record.validate() {
            return Err(format!("Validation error at line {}: {}", line_num + 1, e).into());
        }

        records.push(record);
    }

    Ok(records)
}

pub fn calculate_total(records: &[Record]) -> f64 {
    records.iter()
        .filter(|r| r.active)
        .map(|r| r.value)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,17.3,false").unwrap();
        writeln!(temp_file, "3,Charlie,89.1,true").unwrap();

        let records = parse_csv(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(calculate_total(&records), 131.6);
    }

    #[test]
    fn test_invalid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,,42.5,true").unwrap();

        let result = parse_csv(temp_file.path());
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub fn parse_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid record format at line {}", index + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;

        records.push(Record {
            id,
            name,
            value,
            active,
        });
    }

    Ok(records)
}

pub fn validate_records(records: &[Record]) -> Vec<String> {
    let mut errors = Vec::new();

    for record in records {
        if record.name.is_empty() {
            errors.push(format!("Record {} has empty name", record.id));
        }
        
        if record.value < 0.0 {
            errors.push(format!("Record {} has negative value: {}", record.id, record.value));
        }
        
        if record.id == 0 {
            errors.push(format!("Record has invalid ID: {}", record.id));
        }
    }

    errors
}

pub fn calculate_total(records: &[Record]) -> f64 {
    records.iter()
        .filter(|r| r.active)
        .map(|r| r.value)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test Item,42.5,true").unwrap();
        writeln!(temp_file, "2,Another Item,100.0,false").unwrap();

        let records = parse_csv(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test Item");
        assert_eq!(records[1].value, 100.0);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            Record { id: 1, name: "Valid".to_string(), value: 50.0, active: true },
            Record { id: 2, name: "".to_string(), value: -10.0, active: false },
            Record { id: 0, name: "Test".to_string(), value: 30.0, active: true },
        ];

        let errors = validate_records(&records);
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_calculate_total() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let total = calculate_total(&records);
        assert_eq!(total, 40.0);
    }
}