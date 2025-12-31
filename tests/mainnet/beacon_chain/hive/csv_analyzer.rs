
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
}