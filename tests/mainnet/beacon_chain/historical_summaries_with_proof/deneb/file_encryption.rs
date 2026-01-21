
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let salt_str = SaltString::encode_b64(salt).unwrap();
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .unwrap();
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
    key
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let encrypted_data = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let result = EncryptionResult {
        encrypted_data,
        nonce: nonce_bytes,
        salt,
    };
    
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&result.salt)?;
    output_file.write_all(&result.nonce)?;
    output_file.write_all(&result.encrypted_data)?;
    
    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut encrypted_content = Vec::new();
    file.read_to_end(&mut encrypted_content)?;
    
    if encrypted_content.len() < SALT_SIZE + NONCE_SIZE {
        return Err("File too short to contain salt and nonce".into());
    }
    
    let salt = &encrypted_content[..SALT_SIZE];
    let nonce_bytes = &encrypted_content[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &encrypted_content[SALT_SIZE + NONCE_SIZE..];
    
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let decrypted_data = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, &decrypted_data)?;
    
    Ok(decrypted_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        let enc_result = encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password,
        ).unwrap();
        
        assert_ne!(&enc_result.encrypted_data, plaintext);
        
        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
        ).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password,
        ).unwrap();
        
        let result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
        );
        
        assert!(result.is_err());
    }
}