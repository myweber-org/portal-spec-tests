
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

pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<EncryptionResult, String> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".to_string());
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes: [u8; 12] = OsRng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    cipher
        .encrypt(nonce, data)
        .map(|ciphertext| EncryptionResult {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
        .map_err(|e| format!("Encryption failed: {}", e))
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".to_string());
    }
    if nonce.len() != 12 {
        return Err("Nonce must be 12 bytes".to_string());
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let result = encrypt_data(&data, key)?;

    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    let metadata = format!("NONCE:{}\n", hex::encode(&result.nonce));
    output
        .write_all(metadata.as_bytes())
        .map_err(|e| format!("Failed to write metadata: {}", e))?;
    output
        .write_all(&result.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), String> {
    let content = fs::read(input_path).map_err(|e| format!("Failed to read file: {}", e))?;
    
    let content_str = String::from_utf8_lossy(&content);
    let lines: Vec<&str> = content_str.splitn(2, '\n').collect();
    
    if lines.len() < 2 || !lines[0].starts_with("NONCE:") {
        return Err("Invalid encrypted file format".to_string());
    }
    
    let nonce_hex = &lines[0][6..];
    let nonce = hex::decode(nonce_hex).map_err(|e| format!("Invalid nonce hex: {}", e))?;
    
    let ciphertext = lines[1].as_bytes();
    
    let plaintext = decrypt_data(ciphertext, key, &nonce)?;
    
    fs::write(output_path, plaintext).map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key: [u8; 32] = OsRng.gen();
        let plaintext = b"Secret message for encryption test";
        
        let result = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let key: [u8; 32] = OsRng.gen();
        let test_data = b"Test file content for encryption";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
}