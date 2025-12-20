use sha2::{Digest, Sha256};
use rand::{RngCore, thread_rng};

pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut buffer = vec![0u8; len];
    rng.fill_bytes(&mut buffer);
    buffer
}

pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn sha256_hex_string(data: &[u8]) -> String {
    let hash = sha256_hash(data);
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes_length() {
        let bytes = generate_random_bytes(32);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_sha256_consistent() {
        let data = b"hello world";
        let hash1 = sha256_hash(data);
        let hash2 = sha256_hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_sha256_hex_format() {
        let data = b"test";
        let hex = sha256_hex_string(data);
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }
}