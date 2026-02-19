use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

pub struct HashVerifier;

impl HashVerifier {
    pub fn calculate_file_hash(file_path: &Path) -> Result<String> {
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn verify_file_integrity(file_path: &Path, expected_hash: &str) -> Result<bool> {
        let calculated_hash = Self::calculate_file_hash(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }

    pub fn calculate_string_hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_string_hash(data: &str, expected_hash: &str) -> bool {
        Self::calculate_string_hash(data) == expected_hash.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_hash_verification() {
        let test_data = "Hello, World!";
        let hash = HashVerifier::calculate_string_hash(test_data);
        assert!(HashVerifier::verify_string_hash(test_data, &hash));
    }

    #[test]
    fn test_file_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test file content for hash verification").unwrap();
        
        let file_path = temp_file.path();
        let hash = HashVerifier::calculate_file_hash(file_path).unwrap();
        let verification = HashVerifier::verify_file_integrity(file_path, &hash).unwrap();
        
        assert!(verification);
    }

    #[test]
    fn test_hash_mismatch() {
        let test_data = "Original data";
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!HashVerifier::verify_string_hash(test_data, wrong_hash));
    }
}