
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub numeric_columns: Vec<String>,
    pub text_columns: Vec<String>,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?.clone();
    let column_count = headers.len();
    
    let mut row_count = 0;
    let mut numeric_flags = vec![true; column_count];
    let mut text_flags = vec![false; column_count];
    
    for result in rdr.records() {
        let record = result?;
        row_count += 1;
        
        for (i, field) in record.iter().enumerate() {
            if i >= column_count {
                break;
            }
            
            if numeric_flags[i] {
                if field.parse::<f64>().is_err() && !field.is_empty() {
                    numeric_flags[i] = false;
                    text_flags[i] = true;
                }
            }
        }
    }
    
    let numeric_columns: Vec<String> = headers.iter()
        .enumerate()
        .filter(|(i, _)| numeric_flags[*i])
        .map(|(_, header)| header.to_string())
        .collect();
    
    let text_columns: Vec<String> = headers.iter()
        .enumerate()
        .filter(|(i, _)| text_flags[*i])
        .map(|(_, header)| header.to_string())
        .collect();
    
    Ok(CsvStats {
        row_count,
        column_count,
        numeric_columns,
        text_columns,
    })
}

pub fn filter_csv<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    predicate: impl Fn(&csv::StringRecord) -> bool,
) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;
    
    let mut rdr = csv::Reader::from_reader(input_file);
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
        writeln!(temp_file, "name,age,salary,department").unwrap();
        writeln!(temp_file, "Alice,30,75000.50,Engineering").unwrap();
        writeln!(temp_file, "Bob,25,60000,Sales").unwrap();
        writeln!(temp_file, "Charlie,35,Marketing").unwrap();
        
        let stats = analyze_csv(temp_file.path()).unwrap();
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 4);
        assert_eq!(stats.numeric_columns, vec!["age", "salary"]);
        assert_eq!(stats.text_columns, vec!["name", "department"]);
    }
    
    #[test]
    fn test_filter_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,department").unwrap();
        writeln!(input_file, "Alice,30,Engineering").unwrap();
        writeln!(input_file, "Bob,25,Sales").unwrap();
        writeln!(input_file, "Charlie,35,Marketing").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let filtered = filter_csv(
            input_file.path(),
            output_file.path(),
            |record| record.get(1).and_then(|age| age.parse::<i32>().ok()).unwrap_or(0) >= 30
        ).unwrap();
        
        assert_eq!(filtered, 2);
    }
}