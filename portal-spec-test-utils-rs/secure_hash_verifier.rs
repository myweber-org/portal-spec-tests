
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;

pub enum HashAlgorithm {
    SHA256,
    BLAKE3,
}

pub struct FileHashVerifier {
    algorithm: HashAlgorithm,
}

impl FileHashVerifier {
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn calculate_hash(&self, file_path: &Path) -> io::Result<String> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        match self.algorithm {
            HashAlgorithm::SHA256 => {
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                let result = hasher.finalize();
                Ok(format!("{:x}", result))
            }
            HashAlgorithm::BLAKE3 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(&buffer);
                let result = hasher.finalize();
                Ok(result.to_string())
            }
        }
    }

    pub fn verify_hash(&self, file_path: &Path, expected_hash: &str) -> io::Result<bool> {
        let calculated_hash = self.calculate_hash(file_path)?;
        Ok(calculated_hash == expected_hash)
    }
}

pub fn compare_files_hash(
    file1: &Path,
    file2: &Path,
    algorithm: HashAlgorithm,
) -> io::Result<bool> {
    let verifier = FileHashVerifier::new(algorithm);
    let hash1 = verifier.calculate_hash(file1)?;
    let hash2 = verifier.calculate_hash(file2)?;
    Ok(hash1 == hash2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content for hashing").unwrap();
        
        let verifier = FileHashVerifier::new(HashAlgorithm::SHA256);
        let hash = verifier.calculate_hash(temp_file.path()).unwrap();
        
        assert_eq!(hash.len(), 64);
        assert!(verifier.verify_hash(temp_file.path(), &hash).unwrap());
    }

    #[test]
    fn test_blake3_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Another test content").unwrap();
        
        let verifier = FileHashVerifier::new(HashAlgorithm::BLAKE3);
        let hash = verifier.calculate_hash(temp_file.path()).unwrap();
        
        assert_eq!(hash.len(), 64);
        assert!(verifier.verify_hash(temp_file.path(), &hash).unwrap());
    }

    #[test]
    fn test_file_comparison() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        
        writeln!(file1, "Same content").unwrap();
        writeln!(file2, "Same content").unwrap();
        
        let result = compare_files_hash(
            file1.path(),
            file2.path(),
            HashAlgorithm::SHA256,
        ).unwrap();
        
        assert!(result);
    }
}