use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonObject = serde_json::Map<String, JsonValue>;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = JsonObject::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(obj) = json_data {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(JsonValue::Object(merged))
}

fn merge_objects(target: &mut JsonObject, source: JsonObject) {
    for (key, value) in source {
        match (target.get_mut(&key), value) {
            (Some(JsonValue::Object(existing_obj)), JsonValue::Object(new_obj)) => {
                merge_objects(existing_obj.as_object_mut().unwrap(), new_obj);
            }
            (Some(JsonValue::Array(existing_arr)), JsonValue::Array(new_arr)) => {
                existing_arr.extend(new_arr);
            }
            _ => {
                target.insert(key, value);
            }
        }
    }
}

pub fn merge_json_strings(json_strings: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = JsonObject::new();

    for json_str in json_strings {
        let json_data: JsonValue = serde_json::from_str(json_str)?;

        if let JsonValue::Object(obj) = json_data {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Each JSON string must represent a JSON object".into());
        }
    }

    Ok(JsonValue::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let json1 = json!({
            "name": "Alice",
            "details": {
                "age": 30,
                "city": "London"
            }
        });

        let json2 = json!({
            "details": {
                "country": "UK",
                "age": 31
            },
            "active": true
        });

        let merged = merge_json_strings(&[
            &json1.to_string(),
            &json2.to_string()
        ]).unwrap();

        let expected = json!({
            "name": "Alice",
            "details": {
                "age": 31,
                "city": "London",
                "country": "UK"
            },
            "active": true
        });

        assert_eq!(merged, expected);
    }

    #[test]
    fn test_merge_arrays() {
        let json1 = json!({
            "tags": ["rust", "json"],
            "data": [1, 2]
        });

        let json2 = json!({
            "tags": ["merge"],
            "data": [3, 4]
        });

        let merged = merge_json_strings(&[
            &json1.to_string(),
            &json2.to_string()
        ]).unwrap();

        let expected = json!({
            "tags": ["rust", "json", "merge"],
            "data": [1, 2, 3, 4]
        });

        assert_eq!(merged, expected);
    }
}