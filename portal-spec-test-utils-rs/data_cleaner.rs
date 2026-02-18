
use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
    normalize_case: bool,
}

impl DataCleaner {
    pub fn new(normalize_case: bool) -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
            normalize_case,
        }
    }

    pub fn process(&mut self, input: &str) -> Result<Option<String>, Box<dyn Error>> {
        let mut processed = input.trim().to_string();

        if self.normalize_case {
            processed = processed.to_lowercase();
        }

        if processed.is_empty() {
            return Ok(None);
        }

        if self.dedupe_set.contains(&processed) {
            return Ok(None);
        }

        self.dedupe_set.insert(processed.clone());
        Ok(Some(processed))
    }

    pub fn process_batch(&mut self, inputs: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
        let mut results = Vec::new();
        
        for input in inputs {
            if let Some(processed) = self.process(input)? {
                results.push(processed);
            }
        }
        
        Ok(results)
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }

    pub fn unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new(false);
        
        let result1 = cleaner.process("test").unwrap();
        let result2 = cleaner.process("test").unwrap();
        let result3 = cleaner.process("different").unwrap();
        
        assert_eq!(result1, Some("test".to_string()));
        assert_eq!(result2, None);
        assert_eq!(result3, Some("different".to_string()));
        assert_eq!(cleaner.unique_count(), 2);
    }

    #[test]
    fn test_case_normalization() {
        let mut cleaner = DataCleaner::new(true);
        
        let result1 = cleaner.process("TEST").unwrap();
        let result2 = cleaner.process("test").unwrap();
        
        assert_eq!(result1, Some("test".to_string()));
        assert_eq!(result2, None);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new(false);
        
        let inputs = vec!["apple", "banana", "apple", "cherry", "banana"];
        let results = cleaner.process_batch(&inputs).unwrap();
        
        assert_eq!(results.len(), 3);
        assert!(results.contains(&"apple".to_string()));
        assert!(results.contains(&"banana".to_string()));
        assert!(results.contains(&"cherry".to_string()));
    }
}