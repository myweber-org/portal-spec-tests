
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn calculate_sha256<P: AsRef<Path>>(path: P) -> Result<String, Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn verify_file_integrity<P: AsRef<Path>>(
        file_path: P,
        expected_hash: &str
    ) -> Result<bool, Error> {
        let calculated_hash = Self::calculate_sha256(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }

    pub fn compare_files<P: AsRef<Path>>(
        file1: P,
        file2: P
    ) -> Result<bool, Error> {
        let hash1 = Self::calculate_sha256(&file1)?;
        let hash2 = Self::calculate_sha256(&file2)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content for hashing").unwrap();
        
        let hash = FileHashVerifier::calculate_sha256(temp_file.path())
            .expect("Failed to calculate hash");
        
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_file_comparison() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        
        writeln!(file1, "Same content").unwrap();
        writeln!(file2, "Same content").unwrap();
        
        let result = FileHashVerifier::compare_files(file1.path(), file2.path())
            .expect("Failed to compare files");
        
        assert!(result);
    }

    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test").unwrap();
        
        let hash = FileHashVerifier::calculate_sha256(temp_file.path())
            .expect("Failed to calculate hash");
        
        let verified = FileHashVerifier::verify_file_integrity(
            temp_file.path(),
            &hash
        ).expect("Failed to verify integrity");
        
        assert!(verified);
    }
}