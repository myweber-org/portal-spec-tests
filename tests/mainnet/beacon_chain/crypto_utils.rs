
use sha2::{Digest, Sha256};

pub fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn compute_sha256_hex(data: &[u8]) -> String {
    let hash = compute_sha256(data);
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let data = b"";
        let hash = compute_sha256(data);
        let expected = hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
            .unwrap();
        assert_eq!(hash.as_slice(), expected.as_slice());
    }

    #[test]
    fn test_sha256_hello() {
        let data = b"hello world";
        let hex_hash = compute_sha256_hex(data);
        assert_eq!(hex_hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }
}
use rand::Rng;

pub fn generate_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    password
}

pub fn validate_password_strength(password: &str) -> bool {
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    let length_ok = password.len() >= 12;
    
    has_upper && has_lower && has_digit && has_special && length_ok
}