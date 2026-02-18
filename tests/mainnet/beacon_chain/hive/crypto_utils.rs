
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
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::generate(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < 12 {
        return Err("Ciphertext too short".into());
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_slice, encrypted) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_slice);
    let plaintext = cipher.decrypt(nonce, encrypted)?;
    Ok(plaintext)
}
use rand::Rng;
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_secure_password(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let char_type = rng.gen_range(0..4);
        let c = match char_type {
            0 => rng.gen_range(b'a'..=b'z') as char,
            1 => rng.gen_range(b'A'..=b'Z') as char,
            2 => rng.gen_range(b'0'..=b'9') as char,
            _ => {
                let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
                special_chars.chars().nth(rng.gen_range(0..special_chars.len())).unwrap()
            }
        };
        password.push(c);
    }
    
    password
}