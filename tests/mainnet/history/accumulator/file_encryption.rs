
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const BUFFER_SIZE: usize = 8192;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    let key_len = key.len();
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key_len];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut buffer = [0u8; BUFFER_SIZE];
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        let data_slice = &mut buffer[..bytes_read];
        xor_cipher(data_slice, key);
        
        output_file.write_all(data_slice)?;
    }
    
    output_file.flush()?;
    Ok(())
}

pub fn validate_key(key: &str) -> Result<Vec<u8>, &'static str> {
    if key.is_empty() {
        return Err("Key cannot be empty");
    }
    
    if key.len() < 8 {
        return Err("Key must be at least 8 characters long");
    }
    
    Ok(key.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let original = b"Hello, World!";
        let key = b"secretkey";
        
        let mut encrypted = original.to_vec();
        xor_cipher(&mut encrypted, key);
        
        assert_ne!(encrypted, original);
        
        let mut decrypted = encrypted.clone();
        xor_cipher(&mut decrypted, key);
        
        assert_eq!(decrypted, original);
    }
    
    #[test]
    fn test_file_processing() -> io::Result<()> {
        let test_data = b"Test file content for encryption";
        let key = b"testkey123";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        process_file(input_file.path(), output_file.path(), key)?;
        
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, test_data);
        
        let mut decrypted = encrypted.clone();
        xor_cipher(&mut decrypted, key);
        
        assert_eq!(decrypted, test_data);
        
        Ok(())
    }
    
    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("validkey123").is_ok());
    }
}