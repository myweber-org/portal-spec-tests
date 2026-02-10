use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn normalize_string(s: &str) -> String {
    s.trim().to_lowercase()
}

fn clean_record(record: &mut Record) {
    record.name = normalize_string(&record.name);
    record.category = normalize_string(&record.category);
    
    if record.value < 0.0 {
        record.value = 0.0;
    }
}

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);
    
    for result in csv_reader.deserialize() {
        let mut record: Record = result?;
        clean_record(&mut record);
        csv_writer.serialize(&record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_clean_csv() {
        let input_data = "id,name,value,category\n1,  TEST  ,-5.0,  CATEGORY_A  \n2,Another,42.5,category_b\n";
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        clean_csv(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap()).unwrap();
        
        let mut rdr = csv::Reader::from_path(output_file.path()).unwrap();
        let records: Vec<Record> = rdr.deserialize().map(|r| r.unwrap()).collect();
        
        assert_eq!(records[0].name, "test");
        assert_eq!(records[0].value, 0.0);
        assert_eq!(records[0].category, "category_a");
        
        assert_eq!(records[1].name, "another");
        assert_eq!(records[1].value, 42.5);
        assert_eq!(records[1].category, "category_b");
    }
}