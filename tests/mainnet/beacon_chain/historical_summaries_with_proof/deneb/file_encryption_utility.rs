use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from_slice(&key));
    
    let mut file_content = Vec::new();
    File::open(input_path)?.read_to_end(&mut file_content)?;
    
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    let ciphertext = cipher.encrypt(nonce, file_content.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(&ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut encrypted_data = Vec::new();
    File::open(input_path)?.read_to_end(&mut encrypted_data)?;
    
    if encrypted_data.len() < 32 {
        return Err("Invalid encrypted file format".into());
    }
    
    let (key_bytes, ciphertext) = encrypted_data.split_at(32);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key_bytes));
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
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
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}