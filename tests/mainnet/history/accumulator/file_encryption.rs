
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext).map_err(|e| format!("Failed to read file: {}", e))?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        output_file.write_all(&nonce).map_err(|e| format!("Failed to write nonce: {}", e))?;
        output_file.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        if data.len() < 12 {
            return Err("File too short to contain valid encrypted data".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        output_file.write_all(&plaintext).map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }
}

pub fn generate_key_file(key_path: &Path) -> Result<(), String> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    fs::write(key_path, key.as_slice()).map_err(|e| format!("Failed to write key file: {}", e))?;
    Ok(())
}