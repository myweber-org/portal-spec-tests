use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let nonce = generate_nonce();
    
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err("File too short to contain nonce".to_string());
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;
    
    Ok(())
}

fn derive_key(password: &str) -> aes_gcm::Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for (i, byte) in password_bytes.iter().cycle().take(32).enumerate() {
        key[i] = byte.wrapping_add(i as u8);
    }
    
    aes_gcm::Key::<Aes256Gcm>::from_slice(&key).clone()
}

fn generate_nonce() -> Nonce {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    Nonce::from_slice(&nonce).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let plaintext = "Sensitive data that requires protection";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_content = fs::read_to_string(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, plaintext);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let plaintext = "Secret message";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let result = decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            wrong_password
        );
        
        assert!(result.is_err());
    }
}