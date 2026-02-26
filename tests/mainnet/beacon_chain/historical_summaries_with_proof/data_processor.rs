
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            record.validate()?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: Record) -> Result<(), String> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_records(&self) -> &Vec<Record> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_name = Record::new(2, "".to_string(), 50.0, false);
        assert!(invalid_name.validate().is_err());

        let invalid_value = Record::new(3, "Test".to_string(), -10.0, true);
        assert!(invalid_value.validate().is_err());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();

        let record1 = Record::new(1, "Item1".to_string(), 100.0, true);
        let record2 = Record::new(2, "Item2".to_string(), 200.0, false);

        assert!(processor.add_record(record1.clone()).is_ok());
        assert!(processor.add_record(record2.clone()).is_ok());

        assert_eq!(processor.get_records().len(), 2);
        assert_eq!(processor.filter_active().len(), 1);
        assert_eq!(processor.calculate_total(), 300.0);

        processor.clear();
        assert!(processor.get_records().is_empty());
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        let record1 = Record::new(1, "Alpha".to_string(), 150.0, true);
        let record2 = Record::new(2, "Beta".to_string(), 250.0, false);

        processor.add_record(record1)?;
        processor.add_record(record2)?;

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        processor.save_to_csv(path)?;

        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path)?;

        assert_eq!(new_processor.get_records().len(), 2);
        assert_eq!(new_processor.calculate_total(), 400.0);

        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    pub valid_records: Vec<Vec<String>>,
    pub invalid_records: Vec<Vec<String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            valid_records: Vec::new(),
            invalid_records: Vec::new(),
        }
    }

    pub fn process_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let record: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

            if self.validate_record(&record) {
                self.valid_records.push(record);
            } else {
                self.invalid_records.push(record);
            }
        }

        Ok(())
    }

    fn validate_record(&self, record: &[String]) -> bool {
        if record.len() != 3 {
            return false;
        }

        if record[0].is_empty() || record[1].is_empty() || record[2].is_empty() {
            return false;
        }

        if !record[2].chars().all(|c| c.is_numeric()) {
            return false;
        }

        true
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.valid_records.len(), self.invalid_records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "John,Doe,25").unwrap();
        writeln!(file, "Jane,Smith,30").unwrap();
        writeln!(file, "Invalid,Record,").unwrap();
        writeln!(file, "Another,Bad,ABC").unwrap();

        let mut processor = DataProcessor::new();
        processor.process_csv(file.path().to_str().unwrap()).unwrap();

        let (valid, invalid) = processor.get_stats();
        assert_eq!(valid, 2);
        assert_eq!(invalid, 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataPoint {
    timestamp: String,
    value: f64,
    category: String,
}

impl DataPoint {
    pub fn new(timestamp: String, value: f64, category: String) -> Self {
        DataPoint {
            timestamp,
            value,
            category,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn category(&self) -> &str {
        &self.category
    }
}

pub struct DataProcessor {
    data_points: Vec<DataPoint>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data_points: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.records() {
            let record = result?;
            if record.len() >= 3 {
                let timestamp = record[0].to_string();
                let value: f64 = record[1].parse()?;
                let category = record[2].to_string();
                self.data_points.push(DataPoint::new(timestamp, value, category));
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.data_points.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.data_points.iter().map(|dp| dp.value).sum();
        let count = self.data_points.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data_points
            .iter()
            .map(|dp| (dp.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataPoint> {
        self.data_points
            .iter()
            .filter(|dp| dp.category == category)
            .collect()
    }

    pub fn export_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        csv_writer.write_record(&["timestamp", "value", "category"])?;

        for data_point in &self.data_points {
            csv_writer.write_record(&[
                &data_point.timestamp,
                &data_point.value.to_string(),
                &data_point.category,
            ])?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn add_data_point(&mut self, data_point: DataPoint) {
        self.data_points.push(data_point);
    }

    pub fn data_count(&self) -> usize {
        self.data_points.len()
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
        
        let dp1 = DataPoint::new("2024-01-01 10:00:00".to_string(), 42.5, "A".to_string());
        let dp2 = DataPoint::new("2024-01-01 11:00:00".to_string(), 38.2, "B".to_string());
        
        processor.add_data_point(dp1);
        processor.add_data_point(dp2);
        
        assert_eq!(processor.data_count(), 2);
        
        let (mean, variance, std_dev) = processor.calculate_statistics();
        assert!((mean - 40.35).abs() < 0.01);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }

    #[test]
    fn test_csv_export_import() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_data_point(DataPoint::new("2024-01-01 10:00:00".to_string(), 42.5, "A".to_string()));
        
        let mut temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();
        
        processor.export_to_csv(temp_path)?;
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(temp_path)?;
        
        assert_eq!(new_processor.data_count(), 1);
        Ok(())
    }
}