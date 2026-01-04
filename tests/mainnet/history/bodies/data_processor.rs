
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
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

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn validate_records(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.trim().is_empty() {
                errors.push(format!("Record {} has empty name", index));
            }

            if record.value < 0.0 {
                errors.push(format!("Record {} has negative value: {}", index, record.value));
            }

            if record.category.trim().is_empty() {
                errors.push(format!("Record {} has empty category", index));
            }
        }

        errors
    }

    pub fn get_statistics(&self) -> String {
        let avg = self.calculate_average();
        let max_record = self.find_max_value();
        let validation_errors = self.validate_records();

        let mut stats = String::new();
        stats.push_str(&format!("Total records: {}\n", self.records.len()));

        if let Some(avg_value) = avg {
            stats.push_str(&format!("Average value: {:.2}\n", avg_value));
        }

        if let Some(max) = max_record {
            stats.push_str(&format!("Max value record: ID={}, Value={}\n", max.id, max.value));
        }

        if !validation_errors.is_empty() {
            stats.push_str(&format!("Validation errors: {}\n", validation_errors.len()));
            for error in validation_errors.iter().take(3) {
                stats.push_str(&format!("  - {}\n", error));
            }
        }

        stats
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.3,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.7,CategoryA").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);

        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.1);

        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);

        let errors = processor.validate_records();
        assert!(errors.is_empty());
    }
}