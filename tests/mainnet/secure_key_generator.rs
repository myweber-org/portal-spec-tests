use rand::Rng;
use std::fs::File;
use std::io::Write;

pub struct KeyGenerator {
    rng: rand::rngs::ThreadRng,
}

impl KeyGenerator {
    pub fn new() -> Self {
        KeyGenerator {
            rng: rand::thread_rng(),
        }
    }

    pub fn generate_secure_key(&mut self, length: usize) -> Vec<u8> {
        let mut key = vec![0u8; length];
        self.rng.fill(&mut key[..]);
        key
    }

    pub fn generate_hex_key(&mut self, length: usize) -> String {
        let key = self.generate_secure_key(length);
        hex::encode(key)
    }

    pub fn generate_base64_key(&mut self, length: usize) -> String {
        let key = self.generate_secure_key(length);
        base64::encode(key)
    }

    pub fn save_key_to_file(&self, key: &[u8], filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        file.write_all(key)?;
        Ok(())
    }
}

pub fn generate_encryption_key() -> String {
    let mut generator = KeyGenerator::new();
    generator.generate_hex_key(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_length() {
        let mut gen = KeyGenerator::new();
        let key = gen.generate_secure_key(32);
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_hex_key_format() {
        let mut gen = KeyGenerator::new();
        let hex_key = gen.generate_hex_key(16);
        assert_eq!(hex_key.len(), 32);
        assert!(hex_key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_unique_keys() {
        let mut gen = KeyGenerator::new();
        let key1 = gen.generate_hex_key(32);
        let key2 = gen.generate_hex_key(32);
        assert_ne!(key1, key2);
    }
}