use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

#[derive(Debug)]
pub enum CipherType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    key: &[u8],
    cipher_type: CipherType,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
            let nonce = Nonce::generate(&mut OsRng);
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        CipherType::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
    cipher_type: CipherType,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
            let nonce = Nonce::from_slice(nonce);
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
        CipherType::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
            let nonce = ChaChaNonce::from_slice(nonce);
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_roundtrip() {
        let key = [0x42; 32];
        let plaintext = b"Test encryption data";
        let result = encrypt_data(plaintext, &key, CipherType::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce, CipherType::Aes256Gcm).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_roundtrip() {
        let key = [0x24; 32];
        let plaintext = b"Another test message";
        let result = encrypt_data(plaintext, &key, CipherType::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce, CipherType::ChaCha20Poly1305).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
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

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = fs::read(input_path)?;
        let nonce = Nonce::generate(&mut OsRng);
        
        let encrypted_data = self.cipher.encrypt(&nonce, data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = nonce.to_vec();
        output.extend(encrypted_data);
        
        fs::write(output_path, output)?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted_data = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let encryptor = FileEncryptor::new("test_password123").unwrap();
        let test_data = b"Hello, secure world!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}