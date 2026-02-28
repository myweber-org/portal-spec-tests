use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> io::Result<Self> {
        let salt_string = SaltString::encode_b64(salt).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Salt encoding failed: {}", e))
        })?;
        
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Key derivation failed: {}", e)))?;
        
        let hash_bytes = password_hash.hash.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "No hash generated")
        })?.as_bytes();
        
        if hash_bytes.len() < 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Derived key too short"));
        }
        
        let key_bytes = &hash_bytes[..32];
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut rng = OsRng;
        let nonce_bytes: [u8; NONCE_SIZE] = rng.gen();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Encryption failed: {}", e)))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce_bytes)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "File too short to contain nonce"));
        }
        
        let nonce_bytes = &encrypted_data[..NONCE_SIZE];
        let ciphertext = &encrypted_data[NONCE_SIZE..];
        
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Decryption failed: {}", e)))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut rng = ArgonRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();
        
        let test_data = b"Hello, this is a secret message!";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_data = Vec::new();
        fs::File::open(decrypted_file.path()).unwrap()
            .read_to_end(&mut decrypted_data).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}