use std::fs::File;
use std::io::{Read, Result};
use sha2::{Sha256, Digest};

pub fn calculate_file_hash(filepath: &str) -> Result<String> {
    let mut file = File::open(filepath)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 4096];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_file_integrity(filepath: &str, expected_hash: &str) -> Result<bool> {
    let calculated_hash = calculate_file_hash(filepath)?;
    Ok(calculated_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hash = calculate_file_hash(temp_file.path().to_str().unwrap())
            .expect("Failed to calculate hash");
        
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test data").unwrap();
        
        let hash = calculate_file_hash(temp_file.path().to_str().unwrap())
            .unwrap();
        
        assert!(verify_file_integrity(temp_file.path().to_str().unwrap(), &hash)
            .unwrap());
    }
}