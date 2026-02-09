
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

pub struct HashVerifier {
    chunk_size: usize,
}

impl HashVerifier {
    pub fn new(chunk_size: usize) -> Self {
        HashVerifier { chunk_size }
    }

    pub fn calculate_file_hash(&self, file_path: &Path) -> Result<String, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; self.chunk_size];

        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| format!("Read error: {}", e))?;
            
            if bytes_read == 0 {
                break;
            }
            
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn verify_hmac(&self, data: &[u8], key: &[u8], expected_hmac: &str) -> bool {
        let mut mac = HmacSha256::new_from_slice(key)
            .expect("HMAC key length validation failed");
        
        mac.update(data);
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        
        hex::encode(code_bytes) == expected_hmac
    }

    pub fn compare_hashes(&self, hash1: &str, hash2: &str) -> bool {
        hash1.eq_ignore_ascii_case(hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_calculation() {
        let verifier = HashVerifier::new(4096);
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "Test data for hashing").unwrap();
        let hash = verifier.calculate_file_hash(temp_file.path()).unwrap();
        
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hmac_verification() {
        let verifier = HashVerifier::new(1024);
        let data = b"important message";
        let key = b"secret-key";
        
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(data);
        let expected_hmac = hex::encode(mac.finalize().into_bytes());
        
        assert!(verifier.verify_hmac(data, key, &expected_hmac));
    }
}