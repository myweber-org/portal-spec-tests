
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct JsonMerger {
    conflict_resolution: ConflictResolution,
}

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

impl JsonMerger {
    pub fn new(resolution: ConflictResolution) -> Self {
        JsonMerger {
            conflict_resolution: resolution,
        }
    }

    pub fn merge_files(&self, path1: &Path, path2: &Path) -> Result<Value, String> {
        let content1 = fs::read_to_string(path1)
            .map_err(|e| format!("Failed to read {}: {}", path1.display(), e))?;
        let content2 = fs::read_to_string(path2)
            .map_err(|e| format!("Failed to read {}: {}", path2.display(), e))?;

        let json1: Value = serde_json::from_str(&content1)
            .map_err(|e| format!("Invalid JSON in {}: {}", path1.display(), e))?;
        let json2: Value = serde_json::from_str(&content2)
            .map_err(|e| format!("Invalid JSON in {}: {}", path2.display(), e))?;

        self.merge_values(&json1, &json2)
    }

    fn merge_values(&self, val1: &Value, val2: &Value) -> Result<Value, String> {
        match (val1, val2) {
            (Value::Object(map1), Value::Object(map2)) => self.merge_objects(map1, map2),
            (Value::Array(arr1), Value::Array(arr2)) => self.merge_arrays(arr1, arr2),
            _ => self.resolve_leaf_conflict(val1, val2),
        }
    }

    fn merge_objects(&self, map1: &Map<String, Value>, map2: &Map<String, Value>) -> Result<Value, String> {
        let mut result = Map::new();
        let all_keys: HashSet<_> = map1.keys().chain(map2.keys()).collect();

        for key in all_keys {
            match (map1.get(key), map2.get(key)) {
                (Some(v1), Some(v2)) => {
                    let merged = self.merge_values(v1, v2)?;
                    result.insert(key.clone(), merged);
                }
                (Some(v), None) | (None, Some(v)) => {
                    result.insert(key.clone(), v.clone());
                }
                (None, None) => unreachable!(),
            }
        }

        Ok(Value::Object(result))
    }

    fn merge_arrays(&self, arr1: &[Value], arr2: &[Value]) -> Result<Value, String> {
        match self.conflict_resolution {
            ConflictResolution::MergeArrays => {
                let mut merged = Vec::with_capacity(arr1.len() + arr2.len());
                merged.extend_from_slice(arr1);
                merged.extend_from_slice(arr2);
                Ok(Value::Array(merged))
            }
            _ => self.resolve_leaf_conflict(&Value::Array(arr1.to_vec()), &Value::Array(arr2.to_vec())),
        }
    }

    fn resolve_leaf_conflict(&self, val1: &Value, val2: &Value) -> Result<Value, String> {
        if val1 == val2 {
            return Ok(val1.clone());
        }

        match self.conflict_resolution {
            ConflictResolution::PreferFirst => Ok(val1.clone()),
            ConflictResolution::PreferSecond => Ok(val2.clone()),
            ConflictResolution::FailOnConflict => Err(format!(
                "Conflict between values: {} and {}",
                val1, val2
            )),
            ConflictResolution::MergeArrays => Err("Cannot merge non-array values".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects_prefer_first() {
        let merger = JsonMerger::new(ConflictResolution::PreferFirst);
        let obj1 = json!({"a": 1, "b": 2});
        let obj2 = json!({"b": 3, "c": 4});
        
        let result = merger.merge_values(&obj1, &obj2).unwrap();
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 2);
        assert_eq!(result["c"], 4);
    }

    #[test]
    fn test_merge_arrays() {
        let merger = JsonMerger::new(ConflictResolution::MergeArrays);
        let arr1 = json!([1, 2, 3]);
        let arr2 = json!([4, 5, 6]);
        
        let result = merger.merge_values(&arr1, &arr2).unwrap();
        assert_eq!(result, json!([1, 2, 3, 4, 5, 6]));
    }
}