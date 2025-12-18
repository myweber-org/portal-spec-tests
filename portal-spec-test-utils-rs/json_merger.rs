
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_value = Value::Object(merged);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;
    
    Ok(())
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        if let Some(existing_value) = target.get_mut(&key) {
            match (existing_value, source_value) {
                (Value::Object(ref mut target_obj), Value::Object(source_obj)) => {
                    merge_objects(target_obj, source_obj);
                }
                (Value::Array(ref mut target_arr), Value::Array(source_arr)) => {
                    target_arr.extend(source_arr);
                    target_arr.sort();
                    target_arr.dedup();
                }
                _ => {
                    *existing_value = source_value;
                }
            }
        } else {
            target.insert(key, source_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        let json1 = json!({
            "name": "test",
            "values": [1, 2],
            "nested": {"a": 1}
        });

        let json2 = json!({
            "version": "1.0",
            "values": [2, 3],
            "nested": {"b": 2}
        });

        fs::write(&file1, serde_json::to_string(&json1).unwrap()).unwrap();
        fs::write(&file2, serde_json::to_string(&json2).unwrap()).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();

        let result_content = fs::read_to_string(output.path()).unwrap();
        let result: Value = serde_json::from_str(&result_content).unwrap();

        assert_eq!(result["name"], "test");
        assert_eq!(result["version"], "1.0");
        assert_eq!(result["values"], json!([1, 2, 3]));
        assert_eq!(result["nested"]["a"], 1);
        assert_eq!(result["nested"]["b"], 2);
    }
}