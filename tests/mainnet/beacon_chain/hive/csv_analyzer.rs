
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub has_header: bool,
    pub sample_data: Vec<Vec<String>>,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P, sample_size: usize) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?.clone();
    let has_header = !headers.is_empty();
    let column_count = headers.len();
    
    let mut row_count = 0;
    let mut sample_data = Vec::with_capacity(sample_size.min(10));
    
    for result in rdr.records() {
        let record = result?;
        row_count += 1;
        
        if sample_data.len() < sample_size {
            let row_data: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            sample_data.push(row_data);
        }
    }
    
    Ok(CsvStats {
        row_count,
        column_count,
        has_header,
        sample_data,
    })
}

pub fn validate_csv_structure<P: AsRef<Path>>(file_path: P, expected_columns: usize) -> Result<bool, Box<dyn Error>> {
    let stats = analyze_csv(file_path, 1)?;
    
    if stats.column_count != expected_columns {
        return Ok(false);
    }
    
    for row in &stats.sample_data {
        if row.len() != expected_columns {
            return Ok(false);
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let stats = analyze_csv(temp_file.path(), 2).unwrap();
        
        assert_eq!(stats.row_count, 2);
        assert_eq!(stats.column_count, 3);
        assert!(stats.has_header);
        assert_eq!(stats.sample_data.len(), 2);
    }
    
    #[test]
    fn test_validate_csv_structure() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "col1,col2,col3").unwrap();
        writeln!(temp_file, "a,b,c").unwrap();
        
        let valid = validate_csv_structure(temp_file.path(), 3).unwrap();
        assert!(valid);
        
        let invalid = validate_csv_structure(temp_file.path(), 2).unwrap();
        assert!(!invalid);
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_types: HashMap<String, String>,
    pub numeric_columns: Vec<String>,
    pub text_columns: Vec<String>,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header_line = lines.next()
        .ok_or("Empty CSV file")??;
    let headers: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    let mut column_samples: HashMap<String, Vec<String>> = HashMap::new();
    let mut row_count = 0;
    
    for line_result in lines {
        let line = line_result?;
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if values.len() != headers.len() {
            continue;
        }
        
        for (i, value) in values.iter().enumerate() {
            column_samples
                .entry(headers[i].clone())
                .or_insert_with(Vec::new)
                .push(value.to_string());
        }
        
        row_count += 1;
    }
    
    let mut column_types = HashMap::new();
    let mut numeric_columns = Vec::new();
    let mut text_columns = Vec::new();
    
    for (header, samples) in column_samples {
        let sample_count = samples.len().min(100);
        let mut is_numeric = true;
        
        for sample in samples.iter().take(sample_count) {
            if sample.parse::<f64>().is_err() && !sample.is_empty() {
                is_numeric = false;
                break;
            }
        }
        
        let col_type = if is_numeric { "numeric" } else { "text" };
        column_types.insert(header.clone(), col_type.to_string());
        
        if is_numeric {
            numeric_columns.push(header);
        } else {
            text_columns.push(header);
        }
    }
    
    Ok(CsvStats {
        row_count,
        column_count: headers.len(),
        column_types,
        numeric_columns,
        text_columns,
    })
}

pub fn filter_csv_rows(
    file_path: &str,
    column_name: &str,
    filter_value: &str,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header_line = lines.next()
        .ok_or("Empty CSV file")??;
    let headers: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    let column_index = headers.iter()
        .position(|h| h == column_name)
        .ok_or(format!("Column '{}' not found", column_name))?;
    
    let mut filtered_rows = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        let values: Vec<String> = line.split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        if values.len() > column_index && values[column_index] == filter_value {
            filtered_rows.push(values);
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
        writeln!(temp_file, "name,age,salary,department").unwrap();
        writeln!(temp_file, "Alice,30,50000.0,Engineering").unwrap();
        writeln!(temp_file, "Bob,25,45000.0,Sales").unwrap();
        writeln!(temp_file, "Charlie,35,60000.0,Engineering").unwrap();
        
        let stats = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 4);
        assert_eq!(stats.numeric_columns.len(), 2);
        assert_eq!(stats.text_columns.len(), 2);
    }
    
    #[test]
    fn test_filter_csv_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,department").unwrap();
        writeln!(temp_file, "Alice,30,Engineering").unwrap();
        writeln!(temp_file, "Bob,25,Sales").unwrap();
        writeln!(temp_file, "Charlie,35,Engineering").unwrap();
        
        let filtered = filter_csv_rows(
            temp_file.path().to_str().unwrap(),
            "department",
            "Engineering"
        ).unwrap();
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Charlie");
    }
}