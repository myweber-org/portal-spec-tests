use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_enabled: bool,
    normalization_rules: Vec<NormalizationRule>,
}

pub struct NormalizationRule {
    pattern: String,
    replacement: String,
    case_sensitive: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplication_enabled: true,
            normalization_rules: Vec::new(),
        }
    }

    pub fn with_deduplication(mut self, enabled: bool) -> Self {
        self.deduplication_enabled = enabled;
        self
    }

    pub fn add_normalization_rule(mut self, pattern: &str, replacement: &str, case_sensitive: bool) -> Self {
        self.normalization_rules.push(NormalizationRule {
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            case_sensitive,
        });
        self
    }

    pub fn clean_data(&self, input: &[String]) -> Vec<String> {
        let mut processed: Vec<String> = input.iter()
            .map(|item| self.apply_normalization(item))
            .collect();

        if self.deduplication_enabled {
            processed = self.deduplicate(&processed);
        }

        processed
    }

    fn apply_normalization(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        for rule in &self.normalization_rules {
            if rule.case_sensitive {
                result = result.replace(&rule.pattern, &rule.replacement);
            } else {
                let pattern_lower = rule.pattern.to_lowercase();
                let text_lower = result.to_lowercase();
                
                if text_lower.contains(&pattern_lower) {
                    result = result.replace(&rule.pattern, &rule.replacement);
                }
            }
        }
        
        result.trim().to_string()
    }

    fn deduplicate(&self, items: &[String]) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_items = Vec::new();
        
        for item in items {
            if seen.insert(item.clone()) {
                unique_items.push(item.clone());
            }
        }
        
        unique_items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new();
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        
        let result = cleaner.clean_data(&input);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"cherry".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new()
            .add_normalization_rule("  ", " ", true)
            .add_normalization_rule("--", "-", true);
        
        let input = vec![
            "hello  world".to_string(),
            "test--data".to_string(),
        ];
        
        let result = cleaner.clean_data(&input);
        assert_eq!(result[0], "hello world");
        assert_eq!(result[1], "test-data");
    }
}