
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_active_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn save_filtered_to_csv(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in filtered {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn find_min_value(&self) -> Option<&DataRecord> {
        self.records
            .iter()
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_data = vec![
            DataRecord { id: 1, category: "A".to_string(), value: 10.5, active: true },
            DataRecord { id: 2, category: "B".to_string(), value: 20.3, active: false },
            DataRecord { id: 3, category: "A".to_string(), value: 15.7, active: true },
        ];

        processor.records = test_data;

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.001);

        let active_records = processor.get_active_records();
        assert_eq!(active_records.len(), 2);

        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 2);

        let min_record = processor.find_min_value().unwrap();
        assert_eq!(min_record.id, 1);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let temp_file = NamedTempFile::new()?;
        let test_path = temp_file.path().to_str().unwrap();

        let mut processor = DataProcessor::new();
        
        let test_data = vec![
            DataRecord { id: 1, category: "Test".to_string(), value: 42.0, active: true },
            DataRecord { id: 2, category: "Test".to_string(), value: 84.0, active: false },
        ];

        processor.records = test_data;

        processor.save_filtered_to_csv(test_path, "Test")?;

        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(test_path)?;

        assert_eq!(new_processor.records.len(), 2);
        
        Ok(())
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(Ok(line)) => line.split(',').map(|s| s.trim().to_string()).collect(),
            _ => return Err("Empty CSV file".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| predicate(&record[column_index]))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;

        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        match operation {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }

    pub fn get_column_stats(&self, column_name: &str) -> Option<(usize, usize)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let non_empty_count = self.records
            .iter()
            .filter(|record| !record[column_index].is_empty())
            .count();
        
        Some((self.records.len(), non_empty_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,age,salary").unwrap();
        writeln!(file, "1,Alice,25,50000").unwrap();
        writeln!(file, "2,Bob,30,60000").unwrap();
        writeln!(file, "3,Charlie,35,75000").unwrap();
        writeln!(file, "4,Diana,25,55000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.headers, vec!["id", "name", "age", "salary"]);
        assert_eq!(processor.records.len(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("age", |age| age == "25");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][1], "Alice");
        assert_eq!(filtered[1][1], "Diana");
    }

    #[test]
    fn test_aggregate_column() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let sum = processor.aggregate_numeric_column("salary", "sum");
        assert_eq!(sum, Some(240000.0));
        
        let avg = processor.aggregate_numeric_column("salary", "avg");
        assert_eq!(avg, Some(60000.0));
    }

    #[test]
    fn test_column_stats() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let stats = processor.get_column_stats("name").unwrap();
        assert_eq!(stats, (4, 4));
    }
}