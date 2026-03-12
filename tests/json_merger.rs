use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonResult = Result<JsonValue, Box<dyn std::error::Error>>;

pub struct JsonMerger {
    data: HashMap<String, JsonValue>,
}

impl JsonMerger {
    pub fn new() -> Self {
        JsonMerger {
            data: HashMap::new(),
        }
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> JsonResult {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;
        
        let filename = path.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.data.insert(filename, json_value);
        Ok(())
    }

    pub fn merge_all(&self) -> JsonValue {
        let mut merged = HashMap::new();
        for (key, value) in &self.data {
            merged.insert(key.clone(), value.clone());
        }
        JsonValue::Object(serde_json::Map::from_iter(merged))
    }

    pub fn save_merged<P: AsRef<Path>>(&self, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let merged = self.merge_all();
        let json_string = serde_json::to_string_pretty(&merged)?;
        std::fs::write(output_path, json_string)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_merger() {
        let mut merger = JsonMerger::new();
        
        let json1 = r#"{"name": "test", "value": 42}"#;
        let json2 = r#"{"enabled": true, "items": ["a", "b"]}"#;
        
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        
        std::fs::write(file1.path(), json1).unwrap();
        std::fs::write(file2.path(), json2).unwrap();

        merger.add_file(file1.path()).unwrap();
        merger.add_file(file2.path()).unwrap();

        let merged = merger.merge_all();
        assert!(merged.is_object());
        
        let output_file = NamedTempFile::new().unwrap();
        merger.save_merged(output_file.path()).unwrap();
        
        let saved_content = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: JsonValue = serde_json::from_str(&saved_content).unwrap();
        assert_eq!(parsed, merged);
    }
}