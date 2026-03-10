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
}use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub struct DataCleaner {
    input_path: String,
    output_path: String,
    trim_whitespace: bool,
    remove_duplicates: bool,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            trim_whitespace: true,
            remove_duplicates: true,
        }
    }

    pub fn set_trim_whitespace(&mut self, value: bool) -> &mut Self {
        self.trim_whitespace = value;
        self
    }

    pub fn set_remove_duplicates(&mut self, value: bool) -> &mut Self {
        self.remove_duplicates = value;
        self
    }

    pub fn clean(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        let output_file = File::create(Path::new(&self.output_path))?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);

        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;

        let mut processed_records = 0;
        let mut seen_records = std::collections::HashSet::new();

        for result in csv_reader.records() {
            let record = result?;
            let mut processed_record = record.clone();

            if self.trim_whitespace {
                processed_record = processed_record
                    .iter()
                    .map(|field| field.trim())
                    .collect::<Vec<_>>();
            }

            if self.remove_duplicates {
                let record_key = processed_record.iter().collect::<Vec<_>>();
                if seen_records.contains(&record_key) {
                    continue;
                }
                seen_records.insert(record_key);
            }

            csv_writer.write_record(&processed_record)?;
            processed_records += 1;
        }

        csv_writer.flush()?;
        Ok(processed_records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cleaner_with_duplicates() -> Result<(), Box<dyn Error>> {
        let mut input_file = NamedTempFile::new()?;
        writeln!(input_file, "name,age,city")?;
        writeln!(input_file, "John,25,New York")?;
        writeln!(input_file, "Jane,30,London")?;
        writeln!(input_file, "John,25,New York")?;
        writeln!(input_file, "  Bob  , 35 , Paris ")?;

        let output_file = NamedTempFile::new()?;
        
        let cleaner = DataCleaner::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        );
        
        let processed = cleaner.clean()?;
        assert_eq!(processed, 3);
        
        let output_content = std::fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("Bob,35,Paris"));
        assert_eq!(output_content.matches("John,25,New York").count(), 1);
        
        Ok(())
    }
}use std::collections::HashSet;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: String,
    pub category: String,
}

pub struct DataCleaner {
    pub records: Vec<DataRecord>,
    pub seen_ids: HashSet<u32>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen_ids: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        if record.value.is_empty() {
            return Err("Value cannot be empty".into());
        }

        if record.category.is_empty() {
            return Err("Category cannot be empty".into());
        }

        if self.seen_ids.contains(&record.id) {
            return Err("Duplicate ID detected".into());
        }

        self.seen_ids.insert(record.id);
        self.records.push(record);
        Ok(())
    }

    pub fn remove_duplicates(&mut self) -> usize {
        let initial_count = self.records.len();
        let mut unique_records = Vec::new();
        let mut seen_combinations = HashSet::new();

        for record in self.records.drain(..) {
            let key = format!("{}-{}", record.value, record.category);
            if !seen_combinations.contains(&key) {
                seen_combinations.insert(key);
                unique_records.push(record);
            }
        }

        self.records = unique_records;
        initial_count - self.records.len()
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn validate_all(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.value.len() > 100 {
                errors.push(format!("Record {}: Value exceeds 100 characters", index));
            }

            if !record.category.chars().all(|c| c.is_alphabetic()) {
                errors.push(format!("Record {}: Category contains non-alphabetic characters", index));
            }
        }

        errors
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        let total = self.records.len();
        let valid = self.records
            .iter()
            .filter(|r| !r.value.is_empty() && !r.category.is_empty())
            .count();

        (total, valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_record() {
        let mut cleaner = DataCleaner::new();
        let record = DataRecord {
            id: 1,
            value: "Test".to_string(),
            category: "A".to_string(),
        };

        assert!(cleaner.add_record(record).is_ok());
        assert_eq!(cleaner.records.len(), 1);
    }

    #[test]
    fn test_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        let record1 = DataRecord {
            id: 1,
            value: "Same".to_string(),
            category: "Cat".to_string(),
        };
        let record2 = DataRecord {
            id: 2,
            value: "Same".to_string(),
            category: "Cat".to_string(),
        };

        cleaner.add_record(record1).unwrap();
        cleaner.add_record(record2).unwrap();
        
        let removed = cleaner.remove_duplicates();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.records.len(), 1);
    }
}