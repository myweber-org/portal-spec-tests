
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

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
    fn test_string_encryption_decryption() {
        let original = "Hello, World!";
        let key = Some(0x55);
        
        let encrypted = encrypt_string(original, key);
        let decrypted = decrypt_string(&encrypted, key);
        
        assert_eq!(original, decrypted);
        assert_ne!(original.as_bytes(), encrypted);
    }
    
    #[test]
    fn test_file_encryption_decryption() {
        let original_content = b"Test file content for encryption";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_content).unwrap();
        
        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap(),
            Some(0x77)
        ).unwrap();
        
        decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
            Some(0x77)
        ).unwrap();
        
        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_default_key() {
        let text = "Default key test";
        let encrypted = encrypt_string(text, None);
        let decrypted = decrypt_string(&encrypted, None);
        
        assert_eq!(text, decrypted);
    }
}