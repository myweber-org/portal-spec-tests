
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: &str) -> Self {
        Self {
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
        if self.key.is_empty() {
            return Err("Encryption key cannot be empty".to_string());
        }

        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        let mut buffer = [0u8; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = input_file.read(&mut buffer)
                .map_err(|e| format!("Failed to read from input file: {}", e))?;

            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            
            for byte in &mut processed_buffer {
                *byte ^= self.key[key_index];
                key_index = (key_index + 1) % self.key.len();
            }

            output_file.write_all(&processed_buffer)
                .map_err(|e| format!("Failed to write to output file: {}", e))?;
        }

        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        self.process_bytes(text.as_bytes(), true)
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted = self.process_bytes(data, false);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_bytes(&self, data: &[u8], is_encrypt: bool) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut key_index = 0;

        for &byte in data {
            let processed_byte = if is_encrypt {
                byte ^ self.key[key_index]
            } else {
                byte ^ self.key[key_index]
            };
            result.push(processed_byte);
            key_index = (key_index + 1) % self.key.len();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let encryptor = FileEncryptor::new("secret_key");
        let original_text = "Hello, World!";
        
        let encrypted = encryptor.encrypt_string(original_text);
        let decrypted = encryptor.decrypt_string(&encrypted);
        
        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let encryptor = FileEncryptor::new("test_key_123");
        let original_content = b"This is a test file content for encryption.";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_empty_key() {
        let encryptor = FileEncryptor::new("");
        let temp_file = NamedTempFile::new().unwrap();
        
        let result = encryptor.encrypt_file(temp_file.path(), temp_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
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
use std::fs;
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_data)?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());

    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let ciphertext = cipher.encrypt(nonce, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(salt.as_bytes())?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 22 {
        return Err("Invalid encrypted file format".into());
    }

    let salt_str = std::str::from_utf8(&encrypted_data[..22])?;
    let salt = SaltString::new(salt_str)?;
    let ciphertext = &encrypted_data[22..];

    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());

    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::File::create(output_path)?.write_all(&plaintext)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Test encryption data";
        let password = "secure_password_123";

        fs::write("test_input.txt", test_data).unwrap();
        
        encrypt_file("test_input.txt", "test_encrypted.bin", password).unwrap();
        decrypt_file("test_encrypted.bin", "test_output.txt", password).unwrap();

        let decrypted_data = fs::read("test_output.txt").unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);

        fs::remove_file("test_input.txt").unwrap_or_default();
        fs::remove_file("test_encrypted.bin").unwrap_or_default();
        fs::remove_file("test_output.txt").unwrap_or_default();
    }
}