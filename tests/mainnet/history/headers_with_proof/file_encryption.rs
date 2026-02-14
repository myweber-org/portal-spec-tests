
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Key error: {}", e))?;
    
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let nonce_bytes: [u8; NONCE_SIZE] = OsRng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output.write_all(&nonce_bytes)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Key error: {}", e))?;
    
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err("File too short to contain nonce".to_string());
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, plaintext)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0u8; 32];
        let test_data = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            &key,
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            &key,
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}