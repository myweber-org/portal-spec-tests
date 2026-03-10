use serde_json::{Map, Value};

pub fn merge_json(a: &mut Value, b: Value) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (key, b_val) in b_map {
                if let Some(a_val) = a_map.get_mut(&key) {
                    merge_json(a_val, b_val);
                } else {
                    a_map.insert(key, b_val);
                }
            }
        }
        (a, b) => *a = b,
    }
}

pub fn merge_json_array(arrays: Vec<Value>) -> Value {
    let mut result = Map::new();
    
    for item in arrays {
        if let Value::Object(map) = item {
            for (key, value) in map {
                if let Some(existing) = result.get_mut(&key) {
                    merge_json(existing, value);
                } else {
                    result.insert(key, value);
                }
            }
        }
    }
    
    Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut a = json!({"a": 1, "b": {"c": 2}});
        let b = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut a, b);
        
        assert_eq!(a, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_array_merge() {
        let arrays = vec![
            json!({"a": 1, "b": {"c": 2}}),
            json!({"b": {"d": 3}, "e": 4}),
            json!({"a": 5, "f": 6})
        ];
        
        let result = merge_json_array(arrays);
        
        assert_eq!(result, json!({"a": 5, "b": {"c": 2, "d": 3}, "e": 4, "f": 6}));
    }
}