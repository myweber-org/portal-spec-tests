
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid number of fields at line {}", line_num + 1).into());
            }
            
            let record = Record {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                active: parts[3].parse()?,
            };
            
            if let Err(e) = record.validate() {
                return Err(format!("Validation error at line {}: {}", line_num + 1, e).into());
            }
            
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.active).collect()
    }

    fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    fn find_by_id(&self, id: u32) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }
}

fn process_csv_data() -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file("data.csv")?;
    
    println!("Total records: {}", processor.records.len());
    println!("Active records: {}", processor.filter_active().len());
    println!("Total value: {:.2}", processor.calculate_total());
    
    if let Some(record) = processor.find_by_id(42) {
        println!("Found record 42: {:?}", record);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            active: true,
        };
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -50.0,
            active: false,
        };
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,value,active")?;
        writeln!(temp_file, "1,Item1,100.5,true")?;
        writeln!(temp_file, "2,Item2,200.0,false")?;
        writeln!(temp_file, "3,Item3,300.75,true")?;
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path())?;
        
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.filter_active().len(), 2);
        assert_eq!(processor.calculate_total(), 601.25);
        assert!(processor.find_by_id(2).is_some());
        
        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

pub fn filter_and_transform_csv(
    input_path: &str,
    output_path: &str,
    filter_column: &str,
    filter_value: &str,
    transform_column: &str,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    for result in csv_reader.records() {
        let record = result?;
        
        if let Some(value) = record.headers().iter().position(|h| h == filter_column) {
            if record.get(value) == Some(filter_value) {
                let mut transformed_record = record.clone();
                
                if let Some(pos) = record.headers().iter().position(|h| h == transform_column) {
                    if let Some(cell_value) = transformed_record.get(pos) {
                        let transformed_value = cell_value.to_uppercase();
                        transformed_record[pos] = transformed_value.as_str();
                    }
                }
                
                csv_writer.write_record(&transformed_record)?;
            }
        }
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_and_transform() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,status,department").unwrap();
        writeln!(input_file, "Alice,active,engineering").unwrap();
        writeln!(input_file, "Bob,inactive,sales").unwrap();
        writeln!(input_file, "Carol,active,marketing").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let result = filter_and_transform_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            "status",
            "active",
            "department",
        );

        assert!(result.is_ok());
    }
}