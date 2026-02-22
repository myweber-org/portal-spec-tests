use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};

const SALT_LENGTH: usize = 16;
const TOKEN_LENGTH: usize = 32;

pub fn generate_salt() -> String {
    let mut rng = thread_rng();
    (0..SALT_LENGTH)
        .map(|_| rng.gen_range(33..127) as u8 as char)
        .collect()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn generate_api_token() -> String {
    let mut rng = thread_rng();
    (0..TOKEN_LENGTH)
        .map(|_| rng.gen_range(33..127) as u8 as char)
        .collect()
}

pub fn verify_password(password: &str, salt: &str, hash: &str) -> bool {
    hash_password(password, salt) == hash
}
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_api_key(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub fn generate_secure_token() -> [u8; 32] {
    let mut token = [0u8; 32];
    thread_rng().fill(&mut token);
    token
}

pub fn format_token_hex(token: &[u8]) -> String {
    token.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_length() {
        let key = generate_api_key(24);
        assert_eq!(key.len(), 24);
        assert!(key.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_secure_token_size() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_hex_formatting() {
        let token = [0xAB, 0xCD, 0xEF];
        let hex = format_token_hex(&token);
        assert_eq!(hex, "abcdef");
    }
}