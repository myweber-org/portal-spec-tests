use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;

pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

pub struct FileHashVerifier {
    algorithm: HashAlgorithm,
}

impl FileHashVerifier {
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn calculate_hash(&self, file_path: &Path) -> Result<String, std::io::Error> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 8192];

        match self.algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(hasher.finalize().to_hex().to_string())
            }
        }
    }

    pub fn verify_hash(&self, file_path: &Path, expected_hash: &str) -> Result<bool, std::io::Error> {
        let calculated_hash = self.calculate_hash(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content for hash verification").unwrap();
        
        let verifier = FileHashVerifier::new(HashAlgorithm::Sha256);
        let hash = verifier.calculate_hash(temp_file.path()).unwrap();
        
        assert_eq!(hash.len(), 64);
        assert!(verifier.verify_hash(temp_file.path(), &hash).unwrap());
    }

    #[test]
    fn test_blake3_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Another test for BLAKE3 hashing").unwrap();
        
        let verifier = FileHashVerifier::new(HashAlgorithm::Blake3);
        let hash = verifier.calculate_hash(temp_file.path()).unwrap();
        
        assert_eq!(hash.len(), 64);
        assert!(verifier.verify_hash(temp_file.path(), &hash).unwrap());
    }
}