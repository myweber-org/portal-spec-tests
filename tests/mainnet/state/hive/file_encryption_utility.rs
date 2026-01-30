use aes_gcm::{
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
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
    nonce: [u8; NONCE_SIZE],
}

impl FileEncryptor {
    pub fn from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        
        Ok(Self {
            cipher,
            nonce,
        })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let ciphertext = self.cipher
            .encrypt(Nonce::from_slice(&self.nonce), plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&self.nonce)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn generate_secure_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123!";
        let encryptor = FileEncryptor::from_password(password).unwrap();
        
        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, String> {
        let mut key = [0u8; 32];
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: key.len(),
        };
        
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key)
            .map_err(|e| format!("Key derivation failed: {}", e))?;
        
        Ok(Self {
            key: Key::<Aes256Gcm>::from_slice(&key).into(),
        })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_LENGTH));
        
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(nonce.as_slice())
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(&self.key);
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
        
        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_LENGTH] {
    generate_random_bytes(SALT_LENGTH).try_into().unwrap()
}

fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt)
            .expect("Failed to create encryptor");
        
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption failed");
        
        let encrypted_size = fs::metadata(encrypted_file.path()).unwrap().len();
        assert!(encrypted_size > 0);
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption failed");
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}