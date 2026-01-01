use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

pub struct HashVerifier {
    key: Option<Vec<u8>>,
}

impl HashVerifier {
    pub fn new(key: Option<&[u8]>) -> Self {
        HashVerifier {
            key: key.map(|k| k.to_vec()),
        }
    }

    pub fn compute_file_hash(&self, file_path: &Path) -> Result<String, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn compute_hmac(&self, data: &[u8]) -> Result<String, String> {
        match &self.key {
            Some(key) => {
                let mut mac = Hmac::<Sha256>::new_from_slice(key)
                    .map_err(|e| format!("Invalid key length: {}", e))?;
                mac.update(data);
                let result = mac.finalize().into_bytes();
                Ok(format!("{:x}", result))
            }
            None => Err("HMAC requires a secret key".to_string()),
        }
    }

    pub fn verify_hash(&self, data: &[u8], expected_hash: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let computed_hash = format!("{:x}", hasher.finalize());
        computed_hash == expected_hash
    }

    pub fn verify_hmac(&self, data: &[u8], expected_hmac: &str) -> Result<bool, String> {
        match self.compute_hmac(data) {
            Ok(computed_hmac) => Ok(computed_hmac == expected_hmac),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_file_hash_consistency() {
        let verifier = HashVerifier::new(None);
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hash1 = verifier.compute_file_hash(temp_file.path()).unwrap();
        let hash2 = verifier.compute_file_hash(temp_file.path()).unwrap();
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_hmac_verification() {
        let key = b"secret-key";
        let verifier = HashVerifier::new(Some(key));
        let data = b"important message";
        
        let hmac = verifier.compute_hmac(data).unwrap();
        let verification = verifier.verify_hmac(data, &hmac).unwrap();
        
        assert!(verification);
    }

    #[test]
    fn test_hash_verification() {
        let verifier = HashVerifier::new(None);
        let data = b"test data";
        let hash = "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9";
        
        assert!(verifier.verify_hash(data, hash));
        assert!(!verifier.verify_hash(b"wrong data", hash));
    }
}