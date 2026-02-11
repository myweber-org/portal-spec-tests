use rand::Rng;
use std::error::Error;

pub struct KeyGenerator {
    rng: rand::rngs::ThreadRng,
}

impl KeyGenerator {
    pub fn new() -> Self {
        KeyGenerator {
            rng: rand::thread_rng(),
        }
    }

    pub fn generate_secure_key(&mut self, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        if length < 16 {
            return Err("Key length must be at least 16 bytes".into());
        }

        if length > 4096 {
            return Err("Key length cannot exceed 4096 bytes".into());
        }

        let mut key = vec![0u8; length];
        self.rng.fill(&mut key[..]);
        
        Ok(key)
    }

    pub fn generate_hex_key(&mut self, byte_length: usize) -> Result<String, Box<dyn Error>> {
        let key = self.generate_secure_key(byte_length)?;
        Ok(hex::encode(key))
    }

    pub fn generate_base64_key(&mut self, byte_length: usize) -> Result<String, Box<dyn Error>> {
        let key = self.generate_secure_key(byte_length)?;
        Ok(base64::encode(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let mut generator = KeyGenerator::new();
        
        let key = generator.generate_secure_key(32).unwrap();
        assert_eq!(key.len(), 32);
        
        let hex_key = generator.generate_hex_key(32).unwrap();
        assert_eq!(hex_key.len(), 64);
        
        let base64_key = generator.generate_base64_key(32).unwrap();
        assert!(!base64_key.is_empty());
    }

    #[test]
    fn test_invalid_length() {
        let mut generator = KeyGenerator::new();
        
        assert!(generator.generate_secure_key(15).is_err());
        assert!(generator.generate_secure_key(4097).is_err());
    }
}