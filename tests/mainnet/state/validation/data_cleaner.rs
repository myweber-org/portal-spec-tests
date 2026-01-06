use std::collections::HashSet;

pub fn deduplicate_vector<T: Eq + std::hash::Hash + Clone>(input: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in input {
        if seen.insert(item) {
            result.push(item.clone());
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_integers() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(deduplicate_vector(&input), expected);
    }

    #[test]
    fn test_deduplicate_strings() {
        let input = vec!["apple", "banana", "apple", "orange", "banana"];
        let expected = vec!["apple", "banana", "orange"];
        assert_eq!(deduplicate_vector(&input), expected);
    }

    #[test]
    fn test_empty_vector() {
        let input: Vec<i32> = vec![];
        let expected: Vec<i32> = vec![];
        assert_eq!(deduplicate_vector(&input), expected);
    }
}