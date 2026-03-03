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
use std::collections::HashMap;

pub struct DataCleaner {
    data: HashMap<String, Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            data: HashMap::new(),
        }
    }

    pub fn add_column(&mut self, column_name: &str, values: Vec<Option<String>>) {
        self.data.insert(column_name.to_string(), values);
    }

    pub fn clean_column(&mut self, column_name: &str) -> Result<Vec<String>, String> {
        match self.data.get_mut(column_name) {
            Some(column_data) => {
                let mut cleaned = Vec::new();
                for value in column_data.iter() {
                    match value {
                        Some(v) => {
                            let trimmed = v.trim().to_string();
                            if !trimmed.is_empty() {
                                cleaned.push(trimmed);
                            }
                        }
                        None => continue,
                    }
                }
                Ok(cleaned)
            }
            None => Err(format!("Column '{}' not found", column_name)),
        }
    }

    pub fn remove_null_rows(&mut self) -> HashMap<String, Vec<String>> {
        let mut result = HashMap::new();
        let row_count = self.get_row_count();

        for row in 0..row_count {
            let mut row_has_null = false;
            for column in self.data.values() {
                if column.get(row).map_or(true, |v| v.is_none()) {
                    row_has_null = true;
                    break;
                }
            }

            if !row_has_null {
                for (col_name, column) in &self.data {
                    if let Some(value) = column.get(row) {
                        if let Some(v) = value {
                            result
                                .entry(col_name.clone())
                                .or_insert_with(Vec::new)
                                .push(v.clone());
                        }
                    }
                }
            }
        }

        result
    }

    fn get_row_count(&self) -> usize {
        self.data.values().next().map_or(0, |v| v.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_column() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_column(
            "names",
            vec![
                Some("  John  ".to_string()),
                Some("".to_string()),
                Some("Jane".to_string()),
                None,
                Some("  Bob  ".to_string()),
            ],
        );

        let cleaned = cleaner.clean_column("names").unwrap();
        assert_eq!(cleaned, vec!["John", "Jane", "Bob"]);
    }

    #[test]
    fn test_remove_null_rows() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_column(
            "id",
            vec![
                Some("1".to_string()),
                Some("2".to_string()),
                None,
                Some("4".to_string()),
            ],
        );
        cleaner.add_column(
            "value",
            vec![
                Some("a".to_string()),
                None,
                Some("c".to_string()),
                Some("d".to_string()),
            ],
        );

        let result = cleaner.remove_null_rows();
        assert_eq!(result.get("id").unwrap(), &vec!["1", "4"]);
        assert_eq!(result.get("value").unwrap(), &vec!["a", "d"]);
    }
}use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn clean_record(&mut self, record: &str) -> Option<String> {
        let trimmed = record.trim().to_string();
        
        if trimmed.is_empty() {
            return None;
        }

        if !self.is_valid_format(&trimmed) {
            return None;
        }

        if self.dedupe_set.contains(&trimmed) {
            return None;
        }

        self.dedupe_set.insert(trimmed.clone());
        Some(trimmed)
    }

    fn is_valid_format(&self, record: &str) -> bool {
        record.len() >= 3 && record.len() <= 255
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }

    pub fn process_batch(&mut self, records: Vec<&str>) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| self.clean_record(record))
            .collect()
    }
}

pub fn validate_email(email: &str) -> Result<(), Box<dyn Error>> {
    if email.is_empty() {
        return Err("Email cannot be empty".into());
    }

    if !email.contains('@') {
        return Err("Email must contain @ symbol".into());
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("Invalid email format".into());
    }

    if !parts[1].contains('.') {
        return Err("Domain must contain a dot".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_record() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.clean_record("  test  "), Some("test".to_string()));
        assert_eq!(cleaner.clean_record(""), None);
        assert_eq!(cleaner.clean_record("ab"), None);
        
        let first = cleaner.clean_record("duplicate");
        let second = cleaner.clean_record("duplicate");
        assert_eq!(first, Some("duplicate".to_string()));
        assert_eq!(second, None);
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn test_process_batch() {
        let mut cleaner = DataCleaner::new();
        let records = vec!["a", "valid", "valid", "toolong".repeat(50).as_str(), ""];
        
        let result = cleaner.process_batch(records);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "valid");
        assert_eq!(cleaner.get_unique_count(), 1);
    }
}