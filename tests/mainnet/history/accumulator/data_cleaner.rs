use std::collections::HashSet;

pub fn clean_data<T: Ord + Clone + std::hash::Hash>(data: &[T]) -> Vec<T> {
    let mut unique_set: HashSet<T> = HashSet::new();
    for item in data {
        unique_set.insert(item.clone());
    }
    
    let mut result: Vec<T> = unique_set.into_iter().collect();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = vec![5, 2, 8, 2, 5, 1, 9];
        let expected = vec![1, 2, 5, 8, 9];
        assert_eq!(clean_data(&input), expected);
    }

    #[test]
    fn test_clean_data_strings() {
        let input = vec!["apple", "banana", "apple", "cherry"];
        let expected = vec!["apple", "banana", "cherry"];
        assert_eq!(clean_data(&input), expected);
    }
}