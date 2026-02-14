use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&key)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < 32 {
        return Err("Invalid encrypted file format".into());
    }

    let (ciphertext, key_slice) = data.split_at(data.len() - 32);
    let key = key_slice.try_into().map_err(|_| "Invalid key length")?;
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        FileEncryptor { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        let nonce = Nonce::generate(&mut OsRng);
        
        let encrypted_data = self.cipher
            .encrypt(&nonce, data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(&nonce)?;
        output.write_all(&encrypted_data)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        
        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce"
            ));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted_data = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    
    let data = fs::read(input_path)?;
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let ciphertext = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(&key)?;
    output.write_all(nonce)?;
    output.write_all(&ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let content = fs::read(input_path)?;
    if content.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key and nonce",
        ));
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&content[..32]);
    let nonce = Nonce::from_slice(&content[32..32 + NONCE_SIZE]);
    let ciphertext = &content[32 + NONCE_SIZE..];
    
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret data that needs protection";
        let plain_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(plain_file.path(), test_data).unwrap();
        
        encrypt_file(
            plain_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        ).unwrap();
        
        let decrypted = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted, test_data);
    }
}