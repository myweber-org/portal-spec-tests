use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);
    
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;
    
    for result in csv_reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| if field.trim().is_empty() { "N/A".to_string() } else { field.to_string() })
            .collect();
        
        csv_writer.write_record(&cleaned_record)?;
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
    fn test_clean_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city\nJohn,25,New York\nJane,,London\nBob,30,").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        clean_csv(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap()).unwrap();
        
        let mut rdr = csv::Reader::from_path(output_file.path()).unwrap();
        let records: Vec<_> = rdr.records().collect();
        assert_eq!(records.len(), 3);
        
        let first_record = &records[0].as_ref().unwrap();
        assert_eq!(first_record[0], "John");
        assert_eq!(first_record[1], "25");
        assert_eq!(first_record[2], "New York");
        
        let second_record = &records[1].as_ref().unwrap();
        assert_eq!(second_record[1], "N/A");
    }
}
use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn normalize(&mut self) -> &mut Self
    where
        T: ToString,
    {
        self.data.sort();
        self
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

pub fn clean_string_data(strings: Vec<String>) -> Vec<String> {
    let mut cleaner = DataCleaner::new(strings);
    cleaner.deduplicate().normalize().into_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let data = vec![1, 2, 2, 3, 3, 3];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3]);
    }

    #[test]
    fn test_normalization() {
        let data = vec![3, 1, 2];
        let mut cleaner = DataCleaner::new(data);
        cleaner.normalize();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3]);
    }

    #[test]
    fn test_string_cleaning() {
        let strings = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let cleaned = clean_string_data(strings);
        assert_eq!(cleaned, vec!["apple", "banana", "cherry"]);
    }
}