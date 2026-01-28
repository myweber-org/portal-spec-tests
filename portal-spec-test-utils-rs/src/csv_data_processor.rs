
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: CsvRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.find_max_value().map(|r| r.value);

        (count, avg, max)
    }

    pub fn add_record(&mut self, record: CsvRecord) {
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

    #[test]
    fn test_csv_processor() {
        let mut processor = CsvProcessor::new();

        let record1 = CsvRecord {
            id: 1,
            name: "Item A".to_string(),
            value: 10.5,
            category: "Electronics".to_string(),
        };

        let record2 = CsvRecord {
            id: 2,
            name: "Item B".to_string(),
            value: 25.0,
            category: "Books".to_string(),
        };

        processor.add_record(record1);
        processor.add_record(record2);

        assert_eq!(processor.records.len(), 2);
        assert!(processor.calculate_average().is_some());
    }

    #[test]
    fn test_filtering() {
        let mut processor = CsvProcessor::new();

        let records = vec![
            CsvRecord {
                id: 1,
                name: "Test1".to_string(),
                value: 100.0,
                category: "A".to_string(),
            },
            CsvRecord {
                id: 2,
                name: "Test2".to_string(),
                value: 200.0,
                category: "B".to_string(),
            },
            CsvRecord {
                id: 3,
                name: "Test3".to_string(),
                value: 300.0,
                category: "A".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record);
        }

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
    }
}