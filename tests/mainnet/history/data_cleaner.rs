
use std::collections::HashMap;

pub struct DataCleaner;

impl DataCleaner {
    pub fn clean_string_vector(data: Vec<Option<String>>) -> Vec<String> {
        data.into_iter()
            .filter_map(|item| item.map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn clean_hashmap(data: HashMap<String, Option<String>>) -> HashMap<String, String> {
        data.into_iter()
            .filter_map(|(key, value)| {
                value
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                    .map(|v| (key, v))
            })
            .collect()
    }

    pub fn remove_null_rows<T>(data: Vec<Vec<Option<T>>>) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        data.into_iter()
            .filter(|row| row.iter().all(|cell| cell.is_some()))
            .map(|row| row.into_iter().filter_map(|cell| cell).collect())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string_vector() {
        let input = vec![
            Some("  hello  ".to_string()),
            None,
            Some("".to_string()),
            Some("world".to_string()),
        ];
        let result = DataCleaner::clean_string_vector(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_clean_hashmap() {
        let mut input = HashMap::new();
        input.insert("key1".to_string(), Some("  value1  ".to_string()));
        input.insert("key2".to_string(), None);
        input.insert("key3".to_string(), Some("".to_string()));

        let result = DataCleaner::clean_hashmap(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("key1").unwrap(), "value1");
    }

    #[test]
    fn test_remove_null_rows() {
        let input = vec![
            vec![Some(1), Some(2)],
            vec![None, Some(3)],
            vec![Some(4), None],
            vec![Some(5), Some(6)],
        ];
        let result = DataCleaner::remove_null_rows(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec![1, 2]);
        assert_eq!(result[1], vec![5, 6]);
    }
}