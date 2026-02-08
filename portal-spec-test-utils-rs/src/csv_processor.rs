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
}use std::error::Error;
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

    pub fn filter_by_column_value(&self, column_index: usize, target_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_rows = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            
            if column_index < columns.len() && columns[column_index] == target_value {
                filtered_rows.push(columns);
            }
        }

        Ok(filtered_rows)
    }

    pub fn count_rows(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        Ok(count)
    }

    pub fn get_column_names(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        
        if let Some(first_line) = reader.lines().next() {
            let line = first_line?;
            let columns: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            Ok(columns)
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_by_column_value() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,30,Paris").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor.filter_by_column_value(1, "30").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][0], "Charlie");
    }

    #[test]
    fn test_count_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header1,header2").unwrap();
        writeln!(temp_file, "value1,value2").unwrap();
        writeln!(temp_file, "value3,value4").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let count = processor.count_rows().unwrap();

        assert_eq!(count, 3);
    }

    #[test]
    fn test_get_column_names() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,email").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let columns = processor.get_column_names().unwrap();

        assert_eq!(columns, vec!["id", "name", "email"]);
    }
}