use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn encrypt_string(text: &str, key: Option<u8>) -> Vec<u8> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    text.bytes()
        .map(|byte| byte ^ encryption_key)
        .collect()
}

pub fn decrypt_string(data: &[u8], key: Option<u8>) -> String {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    data.iter()
        .map(|byte| (byte ^ encryption_key) as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_string_encryption() {
        let original = "Hello, World!";
        let encrypted = encrypt_string(original, Some(0x42));
        let decrypted = decrypt_string(&encrypted, Some(0x42));
        
        assert_eq!(original, decrypted);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original_content = b"Secret data";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), original_content)?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0x99),
        )?;
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x99),
        )?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content, decrypted_content.as_slice());
        
        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    
    let mut input_file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;
    
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    
    let key_path = format!("{}.key", output_path);
    let mut key_file = File::create(&key_path)?;
    key_file.write_all(key.as_slice())?;
    
    let nonce_path = format!("{}.nonce", output_path);
    let mut nonce_file = File::create(&nonce_path)?;
    nonce_file.write_all(nonce.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(
    input_path: &str,
    key_path: &str,
    nonce_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ciphertext = Vec::new();
    File::open(input_path)?.read_to_end(&mut ciphertext)?;
    
    let mut key_bytes = Vec::new();
    File::open(key_path)?.read_to_end(&mut key_bytes)?;
    
    let mut nonce_bytes = Vec::new();
    File::open(nonce_path)?.read_to_end(&mut nonce_bytes)?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&plaintext)?;
    
    Ok(())
}

pub fn generate_key() -> Vec<u8> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    key.as_slice().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption data";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        std::fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        ).unwrap();
        
        let key_path = format!("{}.key", encrypted_file.path().to_str().unwrap());
        let nonce_path = format!("{}.nonce", encrypted_file.path().to_str().unwrap());
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            &key_path,
            &nonce_path,
            decrypted_file.path().to_str().unwrap(),
        ).unwrap();
        
        let decrypted_data = std::fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
        
        std::fs::remove_file(&key_path).ok();
        std::fs::remove_file(&nonce_path).ok();
    }
}