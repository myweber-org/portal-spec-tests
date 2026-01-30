use std::collections::HashMap;

pub struct DataProcessor {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
    transformers: Vec<Box<dyn Fn(HashMap<String, String>) -> HashMap<String, String>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            filters: Vec::new(),
            transformers: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn add_transformer<F>(&mut self, transformer: F)
    where
        F: Fn(HashMap<String, String>) -> HashMap<String, String> + 'static,
    {
        self.transformers.push(Box::new(transformer));
    }

    pub fn process(&self, mut data: HashMap<String, String>) -> Option<HashMap<String, String>> {
        for filter in &self.filters {
            if !filter(&data) {
                return None;
            }
        }

        for transformer in &self.transformers {
            data = transformer(data);
        }

        Some(data)
    }

    pub fn process_batch(&self, batch: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        batch
            .into_iter()
            .filter_map(|item| self.process(item))
            .collect()
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_filter(|data| {
        data.contains_key("id") && !data.get("id").unwrap().is_empty()
    });

    processor.add_transformer(|mut data| {
        data.entry("processed".to_string())
            .or_insert("true".to_string());
        data
    });

    processor.add_transformer(|mut data| {
        if let Some(value) = data.get_mut("name") {
            *value = value.trim().to_string();
        }
        data
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let processor = create_default_processor();
        
        let mut valid_data = HashMap::new();
        valid_data.insert("id".to_string(), "123".to_string());
        valid_data.insert("name".to_string(), "  test  ".to_string());
        
        let result = processor.process(valid_data);
        assert!(result.is_some());
        
        let processed = result.unwrap();
        assert_eq!(processed.get("id").unwrap(), "123");
        assert_eq!(processed.get("name").unwrap(), "test");
        assert_eq!(processed.get("processed").unwrap(), "true");
    }

    #[test]
    fn test_invalid_data() {
        let processor = create_default_processor();
        
        let mut invalid_data = HashMap::new();
        invalid_data.insert("name".to_string(), "test".to_string());
        
        let result = processor.process(invalid_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_batch_processing() {
        let processor = create_default_processor();
        
        let mut data1 = HashMap::new();
        data1.insert("id".to_string(), "1".to_string());
        data1.insert("name".to_string(), "  a  ".to_string());
        
        let mut data2 = HashMap::new();
        data2.insert("name".to_string(), "b".to_string());
        
        let mut data3 = HashMap::new();
        data3.insert("id".to_string(), "3".to_string());
        data3.insert("name".to_string(), "  c  ".to_string());
        
        let batch = vec![data1, data2, data3];
        let results = processor.process_batch(batch);
        
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.get("processed").unwrap() == "true"));
    }
}