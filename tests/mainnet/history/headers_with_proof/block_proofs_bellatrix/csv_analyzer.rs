
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_names: Vec<String>,
    pub column_types: HashMap<String, String>,
    pub missing_values: usize,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header_line = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Err("Empty or invalid CSV file".into()),
    };

    let column_names: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let column_count = column_names.len();

    let mut row_count = 0;
    let mut missing_values = 0;
    let mut column_samples: HashMap<String, Vec<String>> = HashMap::new();

    for column in &column_names {
        column_samples.insert(column.clone(), Vec::new());
    }

    for line_result in lines {
        let line = line_result?;
        row_count += 1;

        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if values.len() != column_count {
            return Err(format!("Row {} has {} columns, expected {}", 
                row_count, values.len(), column_count).into());
        }

        for (i, value) in values.iter().enumerate() {
            let column_name = &column_names[i];
            
            if value.is_empty() {
                missing_values += 1;
            } else {
                if let Some(samples) = column_samples.get_mut(column_name) {
                    if samples.len() < 5 {
                        samples.push(value.to_string());
                    }
                }
            }
        }
    }

    let column_types = infer_column_types(&column_samples);

    Ok(CsvStats {
        row_count,
        column_count,
        column_names,
        column_types,
        missing_values,
    })
}

fn infer_column_types(samples: &HashMap<String, Vec<String>>) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for (column, values) in samples {
        if values.is_empty() {
            types.insert(column.clone(), "unknown".to_string());
            continue;
        }

        let mut is_numeric = true;
        let mut is_integer = true;
        let mut has_decimal = false;

        for value in values {
            if let Ok(num) = value.parse::<f64>() {
                if num.fract() != 0.0 {
                    has_decimal = true;
                }
            } else {
                is_numeric = false;
                break;
            }
        }

        let type_str = if !is_numeric {
            "string"
        } else if is_integer && !has_decimal {
            "integer"
        } else {
            "float"
        };

        types.insert(column.clone(), type_str.to_string());
    }

    types
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_analyze_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,salary").unwrap();
        writeln!(temp_file, "1,Alice,30,50000.5").unwrap();
        writeln!(temp_file, "2,Bob,25,45000").unwrap();
        writeln!(temp_file, "3,Charlie,35,").unwrap();

        let stats = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 4);
        assert_eq!(stats.missing_values, 1);
        assert_eq!(stats.column_names, vec!["id", "name", "age", "salary"]);
        assert_eq!(stats.column_types.get("id").unwrap(), "integer");
        assert_eq!(stats.column_types.get("name").unwrap(), "string");
        assert_eq!(stats.column_types.get("age").unwrap(), "integer");
        assert_eq!(stats.column_types.get("salary").unwrap(), "float");
    }

    #[test]
    fn test_analyze_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = analyze_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}