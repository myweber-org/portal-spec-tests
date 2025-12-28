use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
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

pub fn parse_csv<P: AsRef<Path>>(
    path: P,
    config: &CsvConfig,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut lines = reader.lines().enumerate();

    if config.has_headers {
        if let Some((_, header_line)) = lines.next() {
            let headers = parse_line(&header_line?, config.delimiter);
            println!("Headers: {:?}", headers);
        }
    }

    for (line_num, line_result) in lines {
        let line = line_result?;
        let fields = parse_line(&line, config.delimiter);
        
        if fields.is_empty() {
            return Err(format!("Empty record at line {}", line_num + 1).into());
        }
        
        records.push(fields);
    }

    if records.is_empty() {
        return Err("No data records found".into());
    }

    Ok(records)
}

fn parse_line(line: &str, delimiter: char) -> Vec<String> {
    line.split(delimiter)
        .map(|field| field.trim().to_string())
        .filter(|field| !field.is_empty())
        .collect()
}

pub fn validate_records(records: &[Vec<String>], expected_columns: usize) -> Result<(), String> {
    for (idx, record) in records.iter().enumerate() {
        if record.len() != expected_columns {
            return Err(format!(
                "Record {} has {} columns, expected {}",
                idx + 1,
                record.len(),
                expected_columns
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_csv_with_headers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let config = CsvConfig::default();
        let result = parse_csv(temp_file.path(), &config);
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_parse_csv_without_headers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let config = CsvConfig {
            has_headers: false,
            ..CsvConfig::default()
        };
        
        let result = parse_csv(temp_file.path(), &config);
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_validate_records() {
        let valid_records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["x".to_string(), "y".to_string(), "z".to_string()],
        ];
        
        assert!(validate_records(&valid_records, 3).is_ok());
        
        let invalid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["x".to_string(), "y".to_string(), "z".to_string()],
        ];
        
        assert!(validate_records(&invalid_records, 3).is_err());
    }
}