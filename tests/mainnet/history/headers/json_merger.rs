
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        let resolved = resolve_conflict(&key, existing, &value);
                        merged.insert(key, resolved);
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let output_json = Value::Object(merged);
    let serialized = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, serialized)?;

    Ok(())
}

fn resolve_conflict(key: &str, existing: &Value, new: &Value) -> Value {
    match (existing, new) {
        (Value::Array(arr1), Value::Array(arr2)) => {
            let mut combined = arr1.clone();
            combined.extend(arr2.clone());
            Value::Array(combined)
        },
        (Value::Number(_), Value::Number(n2)) => Value::Number(n2.clone()),
        (Value::String(_), Value::String(s2)) => Value::String(s2.clone()),
        (Value::Bool(_), Value::Bool(b2)) => Value::Bool(*b2),
        _ => {
            eprintln!("Conflict detected for key '{}', using new value", key);
            new.clone()
        }
    }
}