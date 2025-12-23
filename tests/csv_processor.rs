use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvConfig {
    pub delimiter: char,
    pub has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            has_headers: true,
        }
    }
}

pub fn validate_csv_file(path: &Path, config: &CsvConfig) -> Result<usize, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(config.delimiter as u8)
        .has_headers(config.has_headers)
        .from_reader(file);

    let mut record_count = 0;
    for result in rdr.records() {
        let record = result?;
        record_count += 1;
        
        if record.is_empty() {
            return Err("Empty record found".into());
        }
        
        for field in record.iter() {
            if field.trim().is_empty() {
                return Err("Empty field found".into());
            }
        }
    }
    
    if record_count == 0 {
        return Err("CSV file contains no data".into());
    }
    
    Ok(record_count)
}

pub fn extract_column(path: &Path, column_index: usize) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut column_data = Vec::new();

    for result in rdr.records() {
        let record = result?;
        if let Some(field) = record.get(column_index) {
            column_data.push(field.to_string());
        }
    }

    Ok(column_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let config = CsvConfig::default();
        let result = validate_csv_file(temp_file.path(), &config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_validate_empty_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "").unwrap();

        let config = CsvConfig::default();
        let result = validate_csv_file(temp_file.path(), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_column() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let result = extract_column(temp_file.path(), 0).unwrap();
        assert_eq!(result, vec!["John", "Alice"]);
    }
}