
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World! This is a test message.";
        let test_key = Some(0xAA);
        
        let input_tempfile = NamedTempFile::new().unwrap();
        let encrypted_tempfile = NamedTempFile::new().unwrap();
        let decrypted_tempfile = NamedTempFile::new().unwrap();
        
        fs::write(input_tempfile.path(), original_text).unwrap();
        
        encrypt_file(
            input_tempfile.path().to_str().unwrap(),
            encrypted_tempfile.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        let encrypted_data = fs::read(encrypted_tempfile.path()).unwrap();
        assert_ne!(encrypted_data, original_text);
        
        decrypt_file(
            encrypted_tempfile.path().to_str().unwrap(),
            decrypted_tempfile.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_tempfile.path()).unwrap();
        assert_eq!(decrypted_data, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let original_text = b"Testing with default key";
        
        let input_tempfile = NamedTempFile::new().unwrap();
        let encrypted_tempfile = NamedTempFile::new().unwrap();
        let decrypted_tempfile = NamedTempFile::new().unwrap();
        
        fs::write(input_tempfile.path(), original_text).unwrap();
        
        encrypt_file(
            input_tempfile.path().to_str().unwrap(),
            encrypted_tempfile.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            encrypted_tempfile.path().to_str().unwrap(),
            decrypted_tempfile.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_tempfile.path()).unwrap();
        assert_eq!(decrypted_data, original_text);
    }
}