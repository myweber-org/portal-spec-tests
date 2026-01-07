
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub headers: Vec<String>,
    pub numeric_columns: Vec<String>,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
    let column_count = headers.len();
    
    let mut row_count = 0;
    let mut numeric_column_indices = Vec::new();
    
    for result in rdr.records() {
        let record = result?;
        row_count += 1;
        
        if row_count == 1 {
            for (i, field) in record.iter().enumerate() {
                if field.parse::<f64>().is_ok() {
                    numeric_column_indices.push(i);
                }
            }
        }
    }
    
    let numeric_columns: Vec<String> = numeric_column_indices
        .iter()
        .map(|&idx| headers[idx].clone())
        .collect();
    
    Ok(CsvStats {
        row_count,
        column_count,
        headers,
        numeric_columns,
    })
}

pub fn filter_csv<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    predicate: impl Fn(&csv::StringRecord) -> bool,
) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = csv::Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(output_file);
    
    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;
    
    let mut filtered_count = 0;
    
    for result in rdr.records() {
        let record = result?;
        if predicate(&record) {
            wtr.write_record(&record)?;
            filtered_count += 1;
        }
    }
    
    wtr.flush()?;
    Ok(filtered_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary\nAlice,30,50000\nBob,25,45000").unwrap();
        
        let stats = analyze_csv(temp_file.path()).unwrap();
        assert_eq!(stats.row_count, 2);
        assert_eq!(stats.column_count, 3);
        assert_eq!(stats.headers, vec!["name", "age", "salary"]);
        assert_eq!(stats.numeric_columns, vec!["age", "salary"]);
    }
    
    #[test]
    fn test_filter_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,salary\nAlice,30,50000\nBob,25,45000\nCharlie,35,60000").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let filtered = filter_csv(
            input_file.path(),
            output_file.path(),
            |record| record.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age > 30)
        ).unwrap();
        
        assert_eq!(filtered, 1);
    }
}