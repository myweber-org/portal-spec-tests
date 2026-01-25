
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
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_material = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(&key_material.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        let ciphertext = self.cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut ciphertext = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut ciphertext)?;
        
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        let plaintext = self.cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

pub fn generate_key_file(password: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    let key_material = password_hash.hash.ok_or("Hash generation failed")?;
    
    fs::write(output_path, key_material.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();
        
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
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce as ChaChaNonce};
use std::error::Error;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub trait Encryptor {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult, Box<dyn Error>>;
    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub struct AesGcmEncryptor {
    key: [u8; 32],
}

impl AesGcmEncryptor {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self { key }
    }

    pub fn from_key(key: [u8; 32]) -> Self {
        Self { key }
    }
}

impl Encryptor for AesGcmEncryptor {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult, Box<dyn Error>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)?;
        let nonce_bytes: [u8; 12] = OsRng.gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext)?;

        Ok(EncryptionResult {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)?;
        let nonce = Nonce::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}

pub struct ChaChaEncryptor {
    key: [u8; 32],
}

impl ChaChaEncryptor {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self { key }
    }

    pub fn from_key(key: [u8; 32]) -> Self {
        Self { key }
    }
}

impl Encryptor for ChaChaEncryptor {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult, Box<dyn Error>> {
        let key = Key::from_slice(&self.key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce_bytes: [u8; 12] = OsRng.gen();
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext)?;

        Ok(EncryptionResult {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let key = Key::from_slice(&self.key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaChaNonce::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}

pub fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_encryption() {
        let encryptor = AesGcmEncryptor::new();
        let plaintext = b"Secret message for AES-GCM";
        
        let result = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption() {
        let encryptor = ChaChaEncryptor::new();
        let plaintext = b"Secret message for ChaCha20-Poly1305";
        
        let result = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_key_reuse() {
        let key = [42u8; 32];
        let aes_encryptor = AesGcmEncryptor::from_key(key);
        let chacha_encryptor = ChaChaEncryptor::from_key(key);
        
        let plaintext = b"Same key, different algorithms";
        
        let aes_result = aes_encryptor.encrypt(plaintext).unwrap();
        let chacha_result = chacha_encryptor.encrypt(plaintext).unwrap();
        
        let aes_decrypted = aes_encryptor.decrypt(&aes_result.ciphertext, &aes_result.nonce).unwrap();
        let chacha_decrypted = chacha_encryptor.decrypt(&chacha_result.ciphertext, &chacha_result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), aes_decrypted);
        assert_eq!(plaintext.to_vec(), chacha_decrypted);
    }
}