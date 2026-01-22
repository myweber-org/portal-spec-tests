
use rand::rngs::OsRng;
use rand::RngCore;
use std::fmt;

pub struct SecureKey {
    bytes: Vec<u8>,
}

impl SecureKey {
    pub fn new(length: usize) -> Result<Self, &'static str> {
        if length < 16 {
            return Err("Key length must be at least 16 bytes");
        }
        
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        
        Ok(SecureKey { bytes })
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }
}

impl fmt::Display for SecureKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Debug for SecureKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureKey([{} bytes])", self.bytes.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let key = SecureKey::new(32).unwrap();
        assert_eq!(key.as_bytes().len(), 32);
    }
    
    #[test]
    fn test_hex_representation() {
        let key = SecureKey::new(16).unwrap();
        let hex = key.to_hex();
        assert_eq!(hex.len(), 32);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_invalid_length() {
        let result = SecureKey::new(8);
        assert!(result.is_err());
    }
}