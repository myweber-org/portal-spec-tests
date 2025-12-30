use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs::{self, File};
use std::io::{Read, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;
    
    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?.as_bytes();
    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| e.to_string())?;
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_data(encrypted: &EncryptionResult, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted.nonce);
    
    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| e.to_string())
}

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    
    let encrypted = encrypt_data(&buffer, password)?;
    
    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&encrypted.salt).map_err(|e| e.to_string())?;
    output_file.write_all(&encrypted.nonce).map_err(|e| e.to_string())?;
    output_file.write_all(&encrypted.ciphertext).map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    
    let mut salt = [0u8; SALT_LENGTH];
    file.read_exact(&mut salt).map_err(|e| e.to_string())?;
    
    let mut nonce = [0u8; NONCE_LENGTH];
    file.read_exact(&mut nonce).map_err(|e| e.to_string())?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;
    
    let encrypted = EncryptionResult {
        ciphertext,
        salt,
        nonce,
    };
    
    let decrypted = decrypt_data(&encrypted, password)?;
    
    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&decrypted).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let data = b"Test encryption data";
        let password = "secure_password_123";
        
        let encrypted = encrypt_data(data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_encryption() {
        let test_data = b"File encryption test content";
        let password = "file_password_456";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password() {
        let data = b"Sensitive information";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_data(data, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
}