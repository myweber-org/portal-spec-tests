use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0xAA;

pub fn xor_encrypt(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|byte| byte ^ key).collect()
}

pub fn xor_decrypt(data: &[u8], key: u8) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn process_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let processed_data = xor_encrypt(&buffer, encryption_key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_symmetry() {
        let original = b"Hello, World!";
        let key = 0x55;
        
        let encrypted = xor_encrypt(original, key);
        let decrypted = xor_decrypt(&encrypted, key);
        
        assert_eq!(original, decrypted.as_slice());
    }
    
    #[test]
    fn test_file_processing() -> io::Result<()> {
        let test_data = b"Test encryption data";
        let key = 0x77;
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        process_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(key)
        )?;
        
        let processed = fs::read(output_file.path())?;
        let restored = xor_decrypt(&processed, key);
        
        assert_eq!(test_data, restored.as_slice());
        Ok(())
    }
}