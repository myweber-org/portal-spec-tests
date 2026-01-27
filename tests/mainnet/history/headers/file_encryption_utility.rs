use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path, is_encrypt: bool) -> Result<(), String> {
        if !input_path.exists() {
            return Err(format!("Input file does not exist: {:?}", input_path));
        }

        let mut input_file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data = self.xor_transform(&buffer);

        let mut output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen::<u8>()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_symmetry() {
        let cipher = XorCipher::new("test_key_123");
        let original_data = b"Hello, this is a secret message!";
        
        let encrypted = cipher.xor_transform(original_data);
        let decrypted = cipher.xor_transform(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let cipher = XorCipher::new("secure_password");
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "Sensitive data: 42.7, true, 'classified'").unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        cipher.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let original_content = fs::read_to_string(input_file.path()).unwrap();
        let decrypted_content = fs::read_to_string(decrypted_file.path()).unwrap();
        
        assert_eq!(original_content, decrypted_content);
    }

    #[test]
    fn test_empty_key() {
        let cipher = XorCipher::new("");
        let data = b"Some data";
        let transformed = cipher.xor_transform(data);
        assert_eq!(data, transformed.as_slice());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub salt: [u8; SALT_SIZE],
    pub nonce: [u8; NONCE_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt)
        .map_err(|e| format!("Failed to encode salt: {}", e))?;
    
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| format!("Password hashing failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Hash too short for AES-256 key".to_string());
    }
    
    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<EncryptionResult, String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_obj = Nonce::from_slice(&nonce);
    let encrypted_data = cipher
        .encrypt(nonce_obj, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output_file.write_all(&nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output_file.write_all(&encrypted_data)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
    
    Ok(EncryptionResult {
        encrypted_data,
        salt,
        nonce,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<Vec<u8>, String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut encrypted_content = Vec::new();
    file.read_to_end(&mut encrypted_content)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    if encrypted_content.len() < SALT_SIZE + NONCE_SIZE {
        return Err("File too short to contain salt and nonce".to_string());
    }
    
    let salt = &encrypted_content[..SALT_SIZE];
    let nonce = &encrypted_content[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &encrypted_content[SALT_SIZE + NONCE_SIZE..];
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_obj = Nonce::from_slice(nonce);
    let decrypted_data = cipher
        .decrypt(nonce_obj, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&decrypted_data)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(decrypted_data)
}

pub fn encrypt_string(data: &str, password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_obj = Nonce::from_slice(&nonce);
    let encrypted_data = cipher
        .encrypt(nonce_obj, data.as_bytes())
        .map_err(|e| format!("String encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        encrypted_data,
        salt,
        nonce,
    })
}

pub fn decrypt_string(result: &EncryptionResult, password: &str) -> Result<String, String> {
    let key = derive_key(password, &result.salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_obj = Nonce::from_slice(&result.nonce);
    let decrypted_bytes = cipher
        .decrypt(nonce_obj, result.encrypted_data.as_ref())
        .map_err(|e| format!("String decryption failed: {}", e))?;
    
    String::from_utf8(decrypted_bytes)
        .map_err(|e| format!("Invalid UTF-8 in decrypted data: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_string_encryption_decryption() {
        let original = "Sensitive data that needs protection";
        let password = "strong_password_123!";
        
        let encrypted = encrypt_string(original, password).unwrap();
        let decrypted = decrypt_string(&encrypted, password).unwrap();
        
        assert_eq!(original, decrypted);
    }
    
    #[test]
    fn test_file_encryption_decryption() {
        let test_data = b"File content to encrypt";
        let password = "another_secure_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption should succeed");
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let original = "Secret message";
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_string(original, correct_password).unwrap();
        let result = decrypt_string(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
}