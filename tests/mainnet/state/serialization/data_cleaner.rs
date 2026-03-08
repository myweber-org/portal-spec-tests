
use csv::{ReaderBuilder, WriterBuilder};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);
    
    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new().from_writer(output_file);
    
    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;
    
    let mut seen = HashSet::new();
    
    for result in reader.records() {
        let record = result?;
        let key: Vec<String> = record.iter().map(|field| field.to_string()).collect();
        
        if seen.insert(key) {
            writer.write_record(&record)?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_remove_duplicates() -> Result<(), Box<dyn Error>> {
        let mut input_file = NamedTempFile::new()?;
        writeln!(input_file, "id,name,value")?;
        writeln!(input_file, "1,Alice,100")?;
        writeln!(input_file, "2,Bob,200")?;
        writeln!(input_file, "1,Alice,100")?;
        writeln!(input_file, "3,Charlie,300")?;
        
        let output_file = NamedTempFile::new()?;
        
        remove_duplicates(input_file.path().to_str().unwrap(), 
                         output_file.path().to_str().unwrap())?;
        
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(File::open(output_file.path())?);
            
        let records: Vec<_> = reader.records().collect::<Result<_, _>>()?;
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].get(0), Some("1"));
        assert_eq!(records[1].get(0), Some("2"));
        assert_eq!(records[2].get(0), Some("3"));
        
        Ok(())
    }
}