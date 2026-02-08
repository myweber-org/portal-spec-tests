use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn export_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.count_records();
        let max = self.get_max_value()
            .map(|r| r.value)
            .unwrap_or(0.0);
        
        format!(
            "Records: {}, Mean: {:.2}, Max: {:.2}",
            count, mean, max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,A").unwrap();
        writeln!(temp_file, "2,20.3,B").unwrap();
        writeln!(temp_file, "3,15.7,A").unwrap();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.filter_by_category("A").len(), 2);
        assert!(processor.calculate_mean().unwrap() > 0.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<Vec<String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            self.data.push(row);
        }
        
        Ok(())
    }

    pub fn validate_data(&self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        
        let header_len = self.data[0].len();
        for row in &self.data {
            if row.len() != header_len {
                return false;
            }
        }
        
        true
    }

    pub fn get_row_count(&self) -> usize {
        self.data.len()
    }

    pub fn get_column_count(&self) -> Option<usize> {
        self.data.first().map(|row| row.len())
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data
            .iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_row_count(), 2);
        assert_eq!(processor.get_column_count(), Some(3));
        assert!(processor.validate_data());
        
        let filtered = processor.filter_rows(|row| row[1].parse::<i32>().unwrap_or(0) > 26);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Alice");
    }
}