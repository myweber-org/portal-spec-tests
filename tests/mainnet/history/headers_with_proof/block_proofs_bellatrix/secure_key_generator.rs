
use rand::Rng;
use std::error::Error;

const KEY_LENGTH: usize = 32;

pub fn generate_secure_key() -> Result<[u8; KEY_LENGTH], Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; KEY_LENGTH];
    rng.fill(&mut key);
    Ok(key)
}

pub fn generate_hex_key() -> Result<String, Box<dyn Error>> {
    let key = generate_secure_key()?;
    Ok(hex::encode(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_length() {
        let key = generate_secure_key().unwrap();
        assert_eq!(key.len(), KEY_LENGTH);
    }

    #[test]
    fn test_key_uniqueness() {
        let key1 = generate_secure_key().unwrap();
        let key2 = generate_secure_key().unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_hex_representation() {
        let hex_key = generate_hex_key().unwrap();
        assert_eq!(hex_key.len(), KEY_LENGTH * 2);
        assert!(hex_key.chars().all(|c| c.is_ascii_hexdigit()));
    }
}