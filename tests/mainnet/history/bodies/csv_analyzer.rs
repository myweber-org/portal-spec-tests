
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
    
    let mut row_count = 0;
    let mut sample_data = Vec::with_capacity(sample_size);
    
    for result in rdr.records() {
        let record = result?;
        row_count += 1;
        
        if sample_data.len() < sample_size {
            let row_data: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            sample_data.push(row_data);
        }
    }
    
    let column_count = if has_header {
        headers.len()
    } else if let Some(first_row) = sample_data.first() {
        first_row.len()
    } else {
        0
    };
    
    Ok(CsvStats {
        row_count,
        column_count,
        has_header,
        sample_data,
    })
}

pub fn validate_csv_data(stats: &CsvStats) -> Vec<String> {
    let mut issues = Vec::new();
    
    if stats.row_count == 0 {
        issues.push("CSV file contains no data rows".to_string());
    }
    
    if stats.column_count == 0 {
        issues.push("CSV file has no columns".to_string());
    }
    
    for (i, row) in stats.sample_data.iter().enumerate() {
        if row.len() != stats.column_count {
            issues.push(format!("Row {} has {} columns, expected {}", 
                i + 1, row.len(), stats.column_count));
        }
        
        for (j, cell) in row.iter().enumerate() {
            if cell.trim().is_empty() {
                issues.push(format!("Empty cell at row {}, column {}", i + 1, j + 1));
            }
        }
    }
    
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv_with_header() {
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
    fn test_validate_csv_data() {
        let stats = CsvStats {
            row_count: 2,
            column_count: 3,
            has_header: true,
            sample_data: vec![
                vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
                vec!["Bob".to_string(), "".to_string(), "London".to_string()],
            ],
        };
        
        let issues = validate_csv_data(&stats);
        assert!(issues.contains(&"Empty cell at row 2, column 2".to_string()));
    }
}