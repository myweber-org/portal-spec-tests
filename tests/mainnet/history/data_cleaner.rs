
use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_whitespace: bool,
    pub trim_strings: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_duplicates: true,
            normalize_whitespace: true,
            trim_strings: true,
        }
    }

    pub fn clean_dataset(&self, data: Vec<String>) -> Vec<String> {
        let mut processed_data = data;

        if self.trim_strings {
            processed_data = processed_data
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
        }

        if self.normalize_whitespace {
            processed_data = processed_data
                .into_iter()
                .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
                .collect();
        }

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed_data.into_iter().collect();
            processed_data = unique_set.into_iter().collect();
        }

        processed_data
    }

    pub fn clean_with_options(
        &self,
        data: Vec<String>,
        remove_duplicates: bool,
        normalize_whitespace: bool,
        trim_strings: bool,
    ) -> Vec<String> {
        let mut processor = DataCleaner {
            remove_duplicates,
            normalize_whitespace,
            trim_strings,
        };
        processor.clean_dataset(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_duplicates() {
        let cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 3);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
        assert!(cleaned.contains(&"cherry".to_string()));
    }

    #[test]
    fn test_cleaner_normalizes_whitespace() {
        let cleaner = DataCleaner::new();
        let data = vec!["  hello    world  ".to_string(), "data\tprocessing".to_string()];
        let cleaned = cleaner.clean_dataset(data);
        assert!(cleaned.contains(&"hello world".to_string()));
        assert!(cleaned.contains(&"data processing".to_string()));
    }

    #[test]
    fn test_cleaner_with_custom_options() {
        let cleaner = DataCleaner::new();
        let data = vec!["  test  ".to_string(), "  test  ".to_string()];
        let cleaned = cleaner.clean_with_options(data, false, true, true);
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0], "test");
    }
}
pub fn normalize_string(input: &str) -> String {
    input.trim().to_lowercase()
}

pub fn clean_string_vector(strings: Vec<&str>) -> Vec<String> {
    strings
        .iter()
        .map(|s| normalize_string(s))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_string() {
        assert_eq!(normalize_string("  HELLO World  "), "hello world");
        assert_eq!(normalize_string("RUST"), "rust");
        assert_eq!(normalize_string(""), "");
    }

    #[test]
    fn test_clean_string_vector() {
        let input = vec!["  APPLE", "Banana  ", "  CHERRY  "];
        let expected = vec!["apple", "banana", "cherry"];
        assert_eq!(clean_string_vector(input), expected);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = csv::Writer::from_writer(File::create(output_path)?);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value.is_finite() && !record.name.is_empty() {
            let cleaned_record = Record {
                name: record.name.trim().to_string(),
                value: (record.value * 100.0).round() / 100.0,
                category: record.category.to_uppercase(),
                ..record
            };
            
            writer.serialize(cleaned_record)?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

pub fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() 
        && record.value >= 0.0 
        && record.value.is_finite()
        && !record.category.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            category: "CATEGORY".to_string(),
        };
        
        assert!(validate_record(&valid_record));
    }

    #[test]
    fn test_clean_csv_data() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,value,category\n1,test,42.567,category";
        let mut temp_input = NamedTempFile::new()?;
        std::io::write(&mut temp_input, input_data)?;
        
        let temp_output = NamedTempFile::new()?;
        
        clean_csv_data(temp_input.path().to_str().unwrap(), 
                      temp_output.path().to_str().unwrap())?;
        
        Ok(())
    }
}