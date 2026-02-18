
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonObject = serde_json::Map<String, JsonValue>;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, String> {
    let mut merged_object = JsonObject::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: JsonValue = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let JsonValue::Object(obj) = json_value {
            for (key, value) in obj {
                merged_object.insert(key, value);
            }
        } else {
            return Err(format!("JSON root in {} is not an object", path_str));
        }
    }

    Ok(JsonValue::Object(merged_object))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<JsonValue, String> {
    let mut merged_object = JsonObject::new();
    let mut key_sources: HashMap<String, Vec<String>> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: JsonValue = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let JsonValue::Object(obj) = json_value {
            for (key, value) in obj {
                key_sources.entry(key.clone()).or_default().push(path_str.to_string());
                
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        merged_object.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        if !merged_object.contains_key(&key) {
                            merged_object.insert(key, value);
                        }
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = merged_object.get(&key) {
                            if existing.is_object() && value.is_object() {
                                if let (JsonValue::Object(mut existing_obj), JsonValue::Object(new_obj)) = (existing.clone(), value) {
                                    for (k, v) in new_obj {
                                        existing_obj.insert(k, v);
                                    }
                                    merged_object.insert(key, JsonValue::Object(existing_obj));
                                }
                            } else {
                                merged_object.insert(key, value);
                            }
                        } else {
                            merged_object.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err(format!("JSON root in {} is not an object", path_str));
        }
    }

    if let ConflictStrategy::Skip = conflict_strategy {
        let duplicates: Vec<_> = key_sources
            .iter()
            .filter(|(_, sources)| sources.len() > 1)
            .map(|(key, sources)| (key.clone(), sources.clone()))
            .collect();
        
        if !duplicates.is_empty() {
            eprintln!("Warning: Duplicate keys found (skipped):");
            for (key, sources) in duplicates {
                eprintln!("  '{}' from: {}", key, sources.join(", "));
            }
        }
    }

    Ok(JsonValue::Object(merged_object))
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::tempdir;

    #[test]
    fn test_merge_basic() {
        let dir = tempdir().unwrap();
        
        let file1 = dir.path().join("a.json");
        write(&file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        
        let file2 = dir.path().join("b.json");
        write(&file2, r#"{"city": "London", "country": "UK"}"#).unwrap();

        let result = merge_json_files(&[
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ]).unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(obj.len(), 4);
    }

    #[test]
    fn test_conflict_overwrite() {
        let dir = tempdir().unwrap();
        
        let file1 = dir.path().join("a.json");
        write(&file1, r#"{"id": 1, "value": "first"}"#).unwrap();
        
        let file2 = dir.path().join("b.json");
        write(&file2, r#"{"id": 2, "value": "second"}"#).unwrap();

        let result = merge_json_with_strategy(
            &[file1.to_str().unwrap(), file2.to_str().unwrap()],
            ConflictStrategy::Overwrite,
        ).unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 2);
        assert_eq!(obj.get("value").unwrap().as_str().unwrap(), "second");
    }
}