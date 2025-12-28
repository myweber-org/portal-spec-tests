use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

pub struct HashVerifier;

impl HashVerifier {
    pub fn sha256_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn verify_sha256<P: AsRef<Path>>(path: P, expected_hash: &str) -> Result<bool, std::io::Error> {
        let calculated = Self::sha256_file(path)?;
        Ok(calculated == expected_hash.to_lowercase())
    }

    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(key)
            .expect("HMAC key length validation failed");
        mac.update(data);
        format!("{:x}", mac.finalize().into_bytes())
    }

    pub fn verify_hmac(key: &[u8], data: &[u8], expected_mac: &str) -> bool {
        let calculated = Self::hmac_sha256(key, data);
        calculated == expected_mac.to_lowercase()
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
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hash = HashVerifier::sha256_file(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64);
        
        let valid = HashVerifier::verify_sha256(temp_file.path(), &hash).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_hmac_generation() {
        let key = b"secret-key";
        let data = b"message to authenticate";
        
        let hmac = HashVerifier::hmac_sha256(key, data);
        assert_eq!(hmac.len(), 64);
        
        let valid = HashVerifier::verify_hmac(key, data, &hmac);
        assert!(valid);
    }
}