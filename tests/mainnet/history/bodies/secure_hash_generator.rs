use sha2::{Digest, Sha256};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

pub struct SecureHashGenerator {
    salt_length: usize,
    iterations: u32,
}

impl SecureHashGenerator {
    pub fn new(salt_length: usize, iterations: u32) -> Self {
        SecureHashGenerator {
            salt_length,
            iterations,
        }
    }

    pub fn generate_salt(&self) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.salt_length)
            .map(char::from)
            .collect()
    }

    pub fn hash_password(&self, password: &str, salt: &str) -> String {
        let mut combined = format!("{}{}", password, salt);
        let mut hasher = Sha256::new();

        for _ in 0..self.iterations {
            hasher.update(combined.as_bytes());
            combined = format!("{:x}", hasher.finalize_reset());
        }

        combined
    }

    pub fn verify_password(&self, password: &str, salt: &str, expected_hash: &str) -> bool {
        self.hash_password(password, salt) == expected_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let generator = SecureHashGenerator::new(16, 1000);
        let salt = generator.generate_salt();
        let password = "SecurePass123!";
        
        let hash1 = generator.hash_password(password, &salt);
        let hash2 = generator.hash_password(password, &salt);
        
        assert_eq!(hash1, hash2);
        assert!(generator.verify_password(password, &salt, &hash1));
    }

    #[test]
    fn test_salt_uniqueness() {
        let generator = SecureHashGenerator::new(16, 1000);
        let salt1 = generator.generate_salt();
        let salt2 = generator.generate_salt();
        
        assert_ne!(salt1, salt2);
        assert_eq!(salt1.len(), 16);
        assert_eq!(salt2.len(), 16);
    }
}