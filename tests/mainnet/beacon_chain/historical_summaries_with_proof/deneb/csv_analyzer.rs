use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_counts: HashMap<String, usize>,
    pub numeric_columns: Vec<String>,
    pub text_columns: Vec<String>,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header_line = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Err("Empty or invalid CSV file".into()),
    };
    
    let headers: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    let mut column_counts: HashMap<String, usize> = headers
        .iter()
        .map(|header| (header.clone(), 0))
        .collect();
    
    let mut row_count = 0;
    let mut numeric_flags: Vec<bool> = vec![true; headers.len()];
    
    for line_result in lines {
        let line = line_result?;
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if values.len() != headers.len() {
            continue;
        }
        
        row_count += 1;
        
        for (i, value) in values.iter().enumerate() {
            if !value.is_empty() {
                *column_counts.get_mut(&headers[i]).unwrap() += 1;
            }
            
            if numeric_flags[i] && !is_numeric(value) {
                numeric_flags[i] = false;
            }
        }
    }
    
    let (numeric_columns, text_columns): (Vec<String>, Vec<String>) = headers
        .iter()
        .zip(numeric_flags.iter())
        .filter_map(|(header, &is_numeric)| {
            if *column_counts.get(header).unwrap_or(&0) > 0 {
                Some((header.clone(), is_numeric))
            } else {
                None
            }
        })
        .partition(|(_, is_numeric)| *is_numeric);
    
    Ok(CsvStats {
        row_count,
        column_counts,
        numeric_columns: numeric_columns.into_iter().map(|(h, _)| h).collect(),
        text_columns: text_columns.into_iter().map(|(h, _)| h).collect(),
    })
}

fn is_numeric(s: &str) -> bool {
    s.parse::<f64>().is_ok() || s.parse::<i64>().is_ok()
}

pub fn filter_csv_rows(
    file_path: &str,
    column: &str,
    predicate: impl Fn(&str) -> bool,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header_line = lines.next().ok_or("Empty file")??;
    let headers: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    let column_index = headers
        .iter()
        .position(|h| h == column)
        .ok_or_else(|| format!("Column '{}' not found", column))?;
    
    let mut filtered_rows = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        let values: Vec<String> = line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        if values.len() > column_index {
            let cell_value = &values[column_index];
            if predicate(cell_value) {
                filtered_rows.push(values);
            }
        }
    }
    
    Ok(filtered_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,salary").unwrap();
        writeln!(temp_file, "1,Alice,30,50000").unwrap();
        writeln!(temp_file, "2,Bob,25,45000").unwrap();
        writeln!(temp_file, "3,Charlie,35,").unwrap();
        
        let stats = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_counts["id"], 3);
        assert_eq!(stats.column_counts["salary"], 2);
        assert!(stats.numeric_columns.contains(&"id".to_string()));
        assert!(stats.numeric_columns.contains(&"age".to_string()));
        assert!(stats.text_columns.contains(&"name".to_string()));
    }
    
    #[test]
    fn test_filter_csv_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,Alice,30").unwrap();
        writeln!(temp_file, "2,Bob,25").unwrap();
        writeln!(temp_file, "3,Charlie,35").unwrap();
        
        let filtered = filter_csv_rows(
            temp_file.path().to_str().unwrap(),
            "age",
            |age| age.parse::<i32>().unwrap_or(0) > 30
        ).unwrap();
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "3");
        assert_eq!(filtered[0][1], "Charlie");
    }
}