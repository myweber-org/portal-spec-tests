
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error, ErrorKind};
use std::path::Path;

pub struct HashVerifier;

impl HashVerifier {
    pub fn compute_file_hash(file_path: &str) -> Result<String, Error> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "File not found"));
        }

        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn verify_file_hash(file_path: &str, expected_hash: &str) -> Result<bool, Error> {
        let computed_hash = Self::compute_file_hash(file_path)?;
        Ok(computed_hash == expected_hash.to_lowercase())
    }

    pub fn compare_files(file1: &str, file2: &str) -> Result<bool, Error> {
        let hash1 = Self::compute_file_hash(file1)?;
        let hash2 = Self::compute_file_hash(file2)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_consistency() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Test data for hash verification";
        temp_file.write_all(test_data).unwrap();
        
        let hash1 = HashVerifier::compute_file_hash(temp_file.path().to_str().unwrap()).unwrap();
        let hash2 = HashVerifier::compute_file_hash(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_file_comparison() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        
        file1.write_all(b"Same content").unwrap();
        file2.write_all(b"Same content").unwrap();
        
        let result = HashVerifier::compare_files(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap()
        ).unwrap();
        
        assert!(result);
    }

    #[test]
    fn test_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Verification test").unwrap();
        
        let computed_hash = HashVerifier::compute_file_hash(
            temp_file.path().to_str().unwrap()
        ).unwrap();
        
        let is_valid = HashVerifier::verify_file_hash(
            temp_file.path().to_str().unwrap(),
            &computed_hash
        ).unwrap();
        
        assert!(is_valid);
    }
}