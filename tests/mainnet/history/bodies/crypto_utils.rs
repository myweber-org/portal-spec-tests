use bcrypt::{hash, verify, DEFAULT_COST};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_salt(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt = generate_salt(16);
        assert_eq!(salt.len(), 16);
        assert!(salt.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_password_hashing() {
        let password = "secure_password_123";
        let hash_result = hash_password(password);
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        let verify_result = verify_password(password, &hash);
        assert_eq!(verify_result, Ok(true));
        
        let wrong_password = "wrong_password";
        let wrong_verify_result = verify_password(wrong_password, &hash);
        assert_eq!(wrong_verify_result, Ok(false));
    }
}use rand::Rng;
use sha2::{Sha256, Digest};

pub fn generate_salt() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; 32];
    rng.fill(&mut salt);
    salt
}

pub fn hash_password(password: &str, salt: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    hex::encode(result)
}