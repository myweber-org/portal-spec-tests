use std::collections::HashSet;

pub fn clean_data(input: Vec<String>) -> Vec<String> {
    let mut unique_items: HashSet<String> = HashSet::new();
    for item in input {
        unique_items.insert(item);
    }
    
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    
    sorted_items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = vec![
            "banana".to_string(),
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
            "apple".to_string(),
        ];
        
        let result = clean_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}