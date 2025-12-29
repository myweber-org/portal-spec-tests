use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, key: u8) -> io::Result<()> {
    let mut buffer = [0; 1024];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for byte in buffer[..bytes_read].iter_mut() {
            *byte ^= key;
        }
        
        writer.write_all(&buffer[..bytes_read])?;
    }
    
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World!";
        let key = 0x55;
        
        let encrypted: Vec<u8> = original_text.iter()
            .map(|byte| byte ^ key)
            .collect();
        
        let decrypted: Vec<u8> = encrypted.iter()
            .map(|byte| byte ^ key)
            .collect();
        
        assert_eq!(original_text.to_vec(), decrypted);
    }

    #[test]
    fn test_file_operations() -> io::Result<()> {
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let restored_file = NamedTempFile::new()?;
        
        let test_data = b"Test data for encryption";
        fs::write(input_file.path(), test_data)?;
        
        encrypt_file(input_file.path().to_str().unwrap(), 
                    output_file.path().to_str().unwrap(), 
                    Some(0x77))?;
        
        decrypt_file(output_file.path().to_str().unwrap(), 
                    restored_file.path().to_str().unwrap(), 
                    Some(0x77))?;
        
        let restored_data = fs::read(restored_file.path())?;
        assert_eq!(test_data.to_vec(), restored_data);
        
        Ok(())
    }
}