
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

pub fn process_file(input_path: &str, output_path: &str, key: Option<u8>, encrypt: bool) -> io::Result<()> {
    if !Path::new(input_path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Input file '{}' not found", input_path)
        ));
    }
    
    if encrypt {
        encrypt_file(input_path, output_path, key)
    } else {
        decrypt_file(input_path, output_path, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, Rust! This is a test message.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_symmetric_operation() {
        let test_data = b"Test data for encryption";
        
        let temp_input = NamedTempFile::new().unwrap();
        let temp_output = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), test_data).unwrap();
        
        process_file(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
            Some(0x99),
            true
        ).unwrap();
        
        let encrypted = fs::read(temp_output.path()).unwrap();
        assert_ne!(test_data, encrypted.as_slice());
        
        process_file(
            temp_output.path().to_str().unwrap(),
            temp_input.path().to_str().unwrap(),
            Some(0x99),
            false
        ).unwrap();
        
        let decrypted = fs::read(temp_input.path()).unwrap();
        assert_eq!(test_data, decrypted.as_slice());
    }
}