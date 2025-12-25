use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn xor_encrypt(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|byte| byte ^ key).collect()
}

pub fn xor_decrypt(data: &[u8], key: u8) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let processed_data = xor_encrypt(&buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;
    
    Ok(())
}

pub fn validate_key(key: &str) -> Option<u8> {
    if key.len() == 2 {
        u8::from_str_radix(key, 16).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_symmetry() {
        let data = b"Hello, World!";
        let key = 0xCC;
        
        let encrypted = xor_encrypt(data, key);
        let decrypted = xor_decrypt(&encrypted, key);
        
        assert_eq!(data, decrypted.as_slice());
    }
    
    #[test]
    fn test_file_processing() -> io::Result<()> {
        let input_data = b"Test file content";
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(input_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_file(input_file.path(), output_file.path(), DEFAULT_KEY)?;
        
        let mut processed_content = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut processed_content)?;
        
        assert_ne!(input_data, processed_content.as_slice());
        
        let restored = xor_decrypt(&processed_content, DEFAULT_KEY);
        assert_eq!(input_data, restored.as_slice());
        
        Ok(())
    }
    
    #[test]
    fn test_key_validation() {
        assert_eq!(validate_key("FF"), Some(255));
        assert_eq!(validate_key("00"), Some(0));
        assert_eq!(validate_key("1A"), Some(26));
        assert_eq!(validate_key("G"), None);
        assert_eq!(validate_key("123"), None);
    }
}