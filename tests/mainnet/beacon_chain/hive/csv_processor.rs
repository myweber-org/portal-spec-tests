
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !columns.is_empty() {
                records.push(CsvRecord { columns });
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<CsvRecord>, predicate: F) -> Vec<CsvRecord>
    where
        F: Fn(&CsvRecord) -> bool,
    {
        records.into_iter().filter(predicate).collect()
    }

    pub fn print_records(&self, records: &[CsvRecord]) {
        for (i, record) in records.iter().enumerate() {
            println!("Record {}: {:?}", i + 1, record.columns);
        }
    }
}

pub fn process_csv_sample() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(',', true);
    
    let records = processor.parse_file("data/sample.csv")?;
    
    println!("Total records: {}", records.len());
    
    let filtered = processor.filter_records(records, |record| {
        record.columns.len() >= 3 && !record.columns[0].is_empty()
    });
    
    println!("Filtered records: {}", filtered.len());
    processor.print_records(&filtered);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let records = processor.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].columns, vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            CsvRecord { columns: vec!["A".to_string(), "B".to_string()] },
            CsvRecord { columns: vec!["C".to_string()] },
            CsvRecord { columns: vec!["D".to_string(), "E".to_string(), "F".to_string()] },
        ];
        
        let processor = CsvProcessor::new(',', false);
        let filtered = processor.filter_records(records, |r| r.columns.len() > 1);
        
        assert_eq!(filtered.len(), 2);
    }
}