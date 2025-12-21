use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::error::Error;

const NONCE_SIZE: usize = 12;

pub fn encrypt_data(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_SIZE]>());
    
    let mut ciphertext = cipher.encrypt(nonce, plaintext)?;
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(nonce);
    result.append(&mut ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < NONCE_SIZE {
        return Err("Ciphertext too short".into());
    }
    
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(&ciphertext[..NONCE_SIZE]);
    let encrypted_data = &ciphertext[NONCE_SIZE..];
    
    cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn test_encryption_roundtrip() {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        
        let plaintext = b"Secret data that needs protection";
        
        let encrypted = encrypt_data(&key, plaintext).unwrap();
        let decrypted = decrypt_data(&key, &encrypted).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileCipher {
    cipher: Aes256Gcm,
}

impl FileCipher {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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
        let cipher = FileCipher::new();
        let test_data = b"Secret data for encryption test";

        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), test_data).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        cipher
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();

        let decrypted_file = NamedTempFile::new().unwrap();
        cipher
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
}