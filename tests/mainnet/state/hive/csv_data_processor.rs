
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: &str, value: f64, active: bool) -> Self {
        Self {
            id,
            category: category.to_string(),
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.value > 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.records.iter().filter(|r| r.is_valid()).collect();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn get_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_active(&self) -> usize {
        self.records.iter().filter(|r| r.active).count()
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, "A", 10.5, true);
        let record2 = DataRecord::new(2, "B", 20.0, false);
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        assert_eq!(processor.count_active(), 1);
        assert_eq!(processor.filter_by_category("A").len(), 1);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 15.25);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,category,value,active")?;
        writeln!(temp_file, "1,Test,100.5,true")?;
        writeln!(temp_file, "2,Another,50.0,false")?;
        
        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path())?;
        
        assert_eq!(processor.records.len(), 2);
        assert_eq!(processor.count_active(), 1);
        
        Ok(())
    }
}