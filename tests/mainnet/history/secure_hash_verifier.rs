use std::fs::File;
use std::io::{Read, BufReader};
use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn verify_integrity(file_path: &str, expected_sha256: &str, expected_blake3: &str) -> Result<bool, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        
        reader.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let sha256_hash = Self::calculate_sha256(&buffer);
        let blake3_hash = Self::calculate_blake3(&buffer);

        let sha256_matches = sha256_hash == expected_sha256;
        let blake3_matches = blake3_hash == expected_blake3;

        if !sha256_matches || !blake3_matches {
            eprintln!("Hash mismatch detected!");
            eprintln!("Expected SHA-256: {}", expected_sha256);
            eprintln!("Calculated SHA-256: {}", sha256_hash);
            eprintln!("Expected BLAKE3: {}", expected_blake3);
            eprintln!("Calculated BLAKE3: {}", blake3_hash);
        }

        Ok(sha256_matches && blake3_matches)
    }

    fn calculate_sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn calculate_blake3(data: &[u8]) -> String {
        let mut hasher = Blake3Hasher::new();
        hasher.update(data);
        hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Test data for hash verification";
        temp_file.write_all(test_data).unwrap();

        let sha256_hash = "a1b2c3d4e5f678901234567890123456789012345678901234567890123456";
        let blake3_hash = "f1e2d3c4b5a6978098765432109876543210987654321098765432109876";

        let result = FileHashVerifier::verify_integrity(
            temp_file.path().to_str().unwrap(),
            sha256_hash,
            blake3_hash
        );

        assert!(result.is_ok());
    }
}