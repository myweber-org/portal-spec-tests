
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

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let fields: Vec<&str> = line.split(',').collect();
            
            if fields.len() != 4 {
                return Err(CsvError::ParseError(
                    format!("Line {}: expected 4 fields, found {}", line_num + 1, fields.len())
                ));
            }

            let id = fields[0].parse::<u32>()
                .map_err(|e| CsvError::ParseError(
                    format!("Line {}: invalid id '{}': {}", line_num + 1, fields[0], e)
                ))?;

            let name = fields[1].trim().to_string();
            if name.is_empty() {
                return Err(CsvError::ValidationError(
                    format!("Line {}: name cannot be empty", line_num + 1)
                ));
            }

            let value = fields[2].parse::<f64>()
                .map_err(|e| CsvError::ParseError(
                    format!("Line {}: invalid value '{}': {}", line_num + 1, fields[2], e)
                ))?;

            if value < 0.0 {
                return Err(CsvError::ValidationError(
                    format!("Line {}: value cannot be negative", line_num + 1)
                ));
            }

            let active = match fields[3].trim().to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                "false" | "0" | "no" => false,
                _ => return Err(CsvError::ParseError(
                    format!("Line {}: invalid boolean value '{}'", line_num + 1, fields[3])
                )),
            };

            self.records.push(CsvRecord {
                id,
                name,
                value,
                active,
            });
        }

        Ok(())
    }

    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter()
            .map(|record| record.value)
            .sum()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&CsvRecord> {
        self.records.iter()
            .find(|record| record.name == name)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_loading() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,Alice,42.5,true").unwrap();
        writeln!(csv_data, "2,Bob,18.3,false").unwrap();
        writeln!(csv_data, "3,Charlie,100.0,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.calculate_total_value(), 160.8);
        assert_eq!(processor.get_active_records().len(), 2);
    }

    #[test]
    fn test_invalid_csv() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,Alice,invalid,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if columns.get(self.filter_column).map_or(false, |&val| val == self.filter_value) {
                let transformed_line = self.transform_row(&columns);
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    fn transform_row(&self, columns: &[&str]) -> String {
        let mut transformed = columns.to_vec();
        if transformed.len() > 2 {
            transformed[1] = transformed[1].to_uppercase().as_str();
        }
        transformed.join(",")
    }
}

pub fn validate_csv_file(path: &str) -> Result<bool, Box<dyn Error>> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err("File does not exist".into());
    }

    let extension = path_obj.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    Ok(extension.eq_ignore_ascii_case("csv"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let input_content = "id,name,value\n1,test,100\n2,sample,200\n3,test,300";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_content).unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            1,
            "test"
        );

        let result = processor.process();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("TEST"));
        assert!(!output_content.contains("sample"));
    }

    #[test]
    fn test_file_validation() {
        let valid_file = NamedTempFile::new().unwrap();
        let valid_path = valid_file.path().with_extension("csv");
        fs::write(&valid_path, "").unwrap();

        assert!(validate_csv_file(valid_path.to_str().unwrap()).unwrap());

        let invalid_file = NamedTempFile::new().unwrap();
        assert!(!validate_csv_file(invalid_file.path().to_str().unwrap()).unwrap());
    }
}use std::error::Error;
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

    for line_result in reader.lines() {
        line_number += 1;
        let line = line_result?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if fields.len() != 4 {
            return Err(format!("Invalid field count at line {}", line_number).into());
        }

        let id = fields[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = fields[1].to_string();
        
        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = fields[3].parse::<bool>()
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
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,false").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,true").unwrap();

        let records = parse_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.3);
        assert!(!records[1].active);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let total = calculate_total_value(&records);
        assert_eq!(total, 40.0);
    }

    #[test]
    fn test_find_max_value_record() {
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