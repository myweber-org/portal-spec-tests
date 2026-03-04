use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_file(path: &Path, key: &[u8; 32]) -> io::Result<EncryptionResult> {
    let plaintext = fs::read(path)?;
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes: [u8; 12] = OsRng.fill(&mut [0u8; 12]).unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

pub fn decrypt_file(ciphertext: &[u8], nonce: &[u8], key: &[u8; 32]) -> io::Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill(&mut key);
    key
}

pub fn save_encrypted_data(output_path: &Path, result: &EncryptionResult) -> io::Result<()> {
    let mut file = fs::File::create(output_path)?;
    file.write_all(&result.nonce)?;
    file.write_all(&result.ciphertext)?;
    Ok(())
}

pub fn load_encrypted_data(path: &Path) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let data = fs::read(path)?;
    if data.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce"
        ));
    }
    
    let nonce = data[..12].to_vec();
    let ciphertext = data[12..].to_vec();
    
    Ok((nonce, ciphertext))
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

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

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
        let plaintext = fs::read(input_path)?;
        
        let mut nonce = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = self.cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_data = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        output_data.extend_from_slice(&nonce);
        output_data.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output_data)?;
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)?;
        Ok(())
    }
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