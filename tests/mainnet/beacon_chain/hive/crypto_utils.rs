
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
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

pub fn generate_numeric_code(digits: u32) -> u32 {
    let min = 10u32.pow(digits - 1);
    let max = 10u32.pow(digits) - 1;
    thread_rng().gen_range(min..=max)
}
use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};

pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()-_=+";
    
    let mut rng = thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn verify_password(password: &str, salt: &str, hash: &str) -> bool {
    hash_password(password, salt) == hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_string_length() {
        let random = generate_random_string(16);
        assert_eq!(random.len(), 16);
    }

    #[test]
    fn test_password_hashing() {
        let password = "SecurePass123!";
        let salt = "randomsalt";
        let hash = hash_password(password, salt);
        
        assert!(verify_password(password, salt, &hash));
        assert!(!verify_password("WrongPass", salt, &hash));
    }
}