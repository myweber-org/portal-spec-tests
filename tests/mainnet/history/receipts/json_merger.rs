
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct JsonMerger {
    data: HashMap<String, JsonValue>,
}

impl JsonMerger {
    pub fn new() -> Self {
        JsonMerger {
            data: HashMap::new(),
        }
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path_ref = path.as_ref();
        let file_name = path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file = File::open(path_ref)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;
        self.data.insert(file_name, json_data);

        Ok(())
    }

    pub fn add_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<()> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                self.add_file(&path)?;
            }
        }
        Ok(())
    }

    pub fn merge(&self) -> JsonValue {
        let mut merged = JsonValue::Object(serde_json::Map::new());
        for (key, value) in &self.data {
            merged[key] = value.clone();
        }
        merged
    }

    pub fn save_merged<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let merged = self.merge();
        let json_string = serde_json::to_string_pretty(&merged)?;
        fs::write(output_path, json_string)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_merge_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("config.json");
        let file2_path = temp_dir.path().join("data.json");

        fs::write(&file1_path, r#"{"port": 8080, "host": "localhost"}"#).unwrap();
        fs::write(&file2_path, r#"{"users": ["alice", "bob"], "active": true}"#).unwrap();

        let mut merger = JsonMerger::new();
        merger.add_file(&file1_path).unwrap();
        merger.add_file(&file2_path).unwrap();

        let merged = merger.merge();
        assert!(merged.get("config").is_some());
        assert!(merged.get("data").is_some());
        assert_eq!(merged["config"]["port"], 8080);
        assert_eq!(merged["data"]["users"][0], "alice");
    }
}