
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Failed to derive key")?.as_bytes();
        
        if key_bytes.len() < 32 {
            return Err("Key derivation failed: insufficient length".into());
        }
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_content = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_content)?;
        
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, file_content.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let parsed_hash = PasswordHash::new(stored_hash)?;
    let argon2 = Argon2::default();
    
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut content = fs::read(input_path)?;
    xor_cipher(&mut content, key.as_bytes());
    fs::write(output_path, content)
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret";
        let mut data = b"Hello, World!".to_vec();
        let original = data.clone();

        xor_cipher(&mut data, key.as_bytes());
        assert_ne!(data, original);

        xor_cipher(&mut data, key.as_bytes());
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key";
        let temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;

        fs::write(temp_input.path(), b"Test content")?;

        encrypt_file(temp_input.path(), temp_output.path(), key)?;
        let encrypted = fs::read(temp_output.path())?;
        assert_ne!(encrypted, b"Test content");

        let temp_decrypted = NamedTempFile::new()?;
        decrypt_file(temp_output.path(), temp_decrypted.path(), key)?;
        let decrypted = fs::read(temp_decrypted.path())?;
        assert_eq!(decrypted, b"Test content");

        Ok(())
    }
}