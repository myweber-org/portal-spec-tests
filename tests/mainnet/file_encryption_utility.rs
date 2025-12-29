
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path
};
use anyhow::{Context, Result};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
    nonce: Nonce,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(15000, 2, 1, Some(32)).unwrap()
        );
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .context("Failed to hash password")?;
        
        let key_bytes = password_hash.hash.unwrap().as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        Ok(Self { cipher, nonce })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let mut input_file = File::open(input_path)
            .context("Failed to open input file")?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)
            .context("Failed to read input file")?;
        
        let ciphertext = self.cipher
            .encrypt(self.nonce, plaintext.as_ref())
            .context("Encryption failed")?;
        
        let mut output_file = File::create(output_path)
            .context("Failed to create output file")?;
        output_file.write_all(&ciphertext)
            .context("Failed to write encrypted data")?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let mut input_file = File::open(input_path)
            .context("Failed to open encrypted file")?;
        let mut ciphertext = Vec::new();
        input_file.read_to_end(&mut ciphertext)
            .context("Failed to read encrypted file")?;
        
        let plaintext = self.cipher
            .decrypt(self.nonce, ciphertext.as_ref())
            .context("Decryption failed - possibly wrong password")?;
        
        let mut output_file = File::create(output_path)
            .context("Failed to create output file")?;
        output_file.write_all(&plaintext)
            .context("Failed to write decrypted data")?;
        
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
    fn test_encryption_decryption() -> Result<()> {
        let password = "secure_password_123!";
        let encryptor = FileEncryptor::new(password)?;
        
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), original_content)?;
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())?;
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content.as_slice(), decrypted_content.as_slice());
        
        Ok(())
    }
}