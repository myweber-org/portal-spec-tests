use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-encryption-key-2024";

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    let key_len = key.len();
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key_len];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: Option<&[u8]>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut content = fs::read(input_path)?;
    xor_cipher(&mut content, encryption_key);
    
    fs::write(output_path, &content)?;
    Ok(())
}

pub fn validate_key(key: &[u8]) -> bool {
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
        let key = b"test";
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_processing() -> io::Result<()> {
        let input_content = b"Hello, World!";
        let input_file = NamedTempFile::new()?;
        fs::write(input_file.path(), input_content)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_file(input_file.path(), output_file.path(), Some(b"custom-key"))?;
        
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, input_content);
        
        let mut decrypted = encrypted.clone();
        xor_cipher(&mut decrypted, b"custom-key");
        assert_eq!(decrypted, input_content);
        
        Ok(())
    }
}