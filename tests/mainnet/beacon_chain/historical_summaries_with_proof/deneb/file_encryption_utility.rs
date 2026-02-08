
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, ParamsBuilder
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    algorithm: String,
    key_length: usize,
}

impl FileEncryptor {
    pub fn new() -> Self {
        FileEncryptor {
            algorithm: String::from("AES-256-GCM"),
            key_length: 32,
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        file.read_to_end(&mut file_data)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let derived_key = self.derive_key(password)?;
        let cipher = self.create_cipher(&derived_key);
        let nonce = self.generate_nonce();
        
        let encrypted_data = cipher.encrypt(&nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&nonce)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output_file.write_all(&encrypted_data)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted file format".to_string());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let derived_key = self.derive_key(password)?;
        let cipher = self.create_cipher(&derived_key);
        
        let decrypted_data = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&decrypted_data)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }

    fn derive_key(&self, password: &str) -> Result<Vec<u8>, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            ParamsBuilder::new()
                .output_len(self.key_length)
                .build()
                .map_err(|e| format!("Failed to build Argon2 params: {}", e))?
        );

        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Key derivation failed: {}", e))?;

        Ok(password_hash.hash.unwrap().as_bytes().to_vec())
    }

    fn create_cipher(&self, key: &[u8]) -> Aes256Gcm {
        let key_array: [u8; 32] = key.try_into()
            .expect("Key must be 32 bytes");
        let key = Key::<Aes256Gcm>::from_slice(&key_array);
        Aes256Gcm::new(key)
    }

    fn generate_nonce(&self) -> Nonce {
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        Nonce::from_slice(&nonce).to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Test data for encryption and decryption";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        let password = "secure_password_123";
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption should succeed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}