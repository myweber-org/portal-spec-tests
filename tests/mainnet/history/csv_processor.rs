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
    let mut lines = reader.lines();

    if config.has_headers {
        let _headers = lines.next().transpose()?;
    }

    for line_result in lines {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        let fields: Vec<String> = line
            .split(config.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if !fields.is_empty() {
            records.push(fields);
        }
    }

    if records.is_empty() {
        return Err("No valid data found in CSV file".into());
    }

    Ok(records)
}

pub fn validate_csv_data(records: &[Vec<String>]) -> Result<(), String> {
    if records.is_empty() {
        return Err("Empty dataset".to_string());
    }

    let expected_len = records[0].len();
    for (i, row) in records.iter().enumerate() {
        if row.len() != expected_len {
            return Err(format!(
                "Row {} has {} fields, expected {}",
                i + 1,
                row.len(),
                expected_len
            ));
        }

        for (j, field) in row.iter().enumerate() {
            if field.is_empty() {
                return Err(format!("Empty field at row {}, column {}", i + 1, j + 1));
            }
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
    fn test_validate_csv_data() {
        let valid_data = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];

        assert!(validate_csv_data(&valid_data).is_ok());

        let invalid_data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()],
        ];

        assert!(validate_csv_data(&invalid_data).is_err());
    }

    #[test]
    fn test_empty_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();

        let config = CsvConfig::default();
        let result = parse_csv(temp_file.path(), &config);

        assert!(result.is_err());
    }
}