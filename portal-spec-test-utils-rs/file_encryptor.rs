
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn process_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let key_bytes = key.as_bytes();
    let mut buffer = Vec::new();
    
    let mut input_file = fs::File::open(input_path)?;
    input_file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key_bytes);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn validate_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 256
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let mut data = vec![1, 2, 3, 4, 5];
        let original = data.clone();
        let key = b"secret";
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), b"test data").unwrap();
        
        let result = process_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            "mykey123"
        );
        
        assert!(result.is_ok());
        
        let encrypted = fs::read(output_file.path()).unwrap();
        assert_ne!(encrypted, b"test data");
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("valid"));
        assert!(!validate_key(""));
        assert!(validate_key(&"a".repeat(256)));
        assert!(!validate_key(&"a".repeat(257)));
    }
}