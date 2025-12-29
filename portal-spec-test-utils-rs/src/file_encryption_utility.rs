use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| format!("Failed to encode salt: {}", e))?;
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, Some(32))
            .map_err(|e| format!("Invalid Argon2 parameters: {}", e))?
    );
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("Password hashing failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    if hash_bytes.len() != 32 {
        return Err("Invalid hash length".to_string());
    }
    
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&hash_bytes[..32]);
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let key = derive_key_from_password(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let encrypted_data = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        encrypted_data,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_data(
    encrypted_data: &[u8],
    password: &str,
    salt: &[u8; SALT_LENGTH],
    nonce: &[u8; NONCE_LENGTH]
) -> Result<Vec<u8>, String> {
    let key = derive_key_from_password(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<(), String> {
    let data = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let result = encrypt_data(&data, password)?;
    
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output.write_all(&result.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output.write_all(&result.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output.write_all(&result.encrypted_data)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<(), String> {
    let mut input = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut salt = [0u8; SALT_LENGTH];
    input.read_exact(&mut salt)
        .map_err(|e| format!("Failed to read salt: {}", e))?;
    
    let mut nonce = [0u8; NONCE_LENGTH];
    input.read_exact(&mut nonce)
        .map_err(|e| format!("Failed to read nonce: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    input.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read encrypted data: {}", e))?;
    
    let decrypted_data = decrypt_data(&encrypted_data, password, &salt, &nonce)?;
    
    fs::write(output_path, decrypted_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption and decryption data";
        let password = "secure_password_123";
        
        let result = encrypt_data(test_data, password).unwrap();
        let decrypted = decrypt_data(
            &result.encrypted_data,
            password,
            &result.salt,
            &result.nonce
        ).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_operations() {
        let original_content = b"File content to encrypt";
        let password = "file_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password() {
        let test_data = b"Sensitive information";
        let correct_password = "correct_pass";
        let wrong_password = "wrong_pass";
        
        let result = encrypt_data(test_data, correct_password).unwrap();
        
        let decryption_result = decrypt_data(
            &result.encrypted_data,
            wrong_password,
            &result.salt,
            &result.nonce
        );
        
        assert!(decryption_result.is_err());
    }
}