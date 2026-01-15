
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data = xor_transform(&input_data, key);
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

fn xor_transform(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    for (i, &byte) in data.iter().enumerate() {
        let key_byte = key[i % key.len()];
        result.push(byte ^ key_byte);
    }
    result
}

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, key: &[u8]) -> io::Result<()> {
    let mut buffer = [0; 4096];
    let mut position = 0;
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for i in 0..bytes_read {
            buffer[i] ^= key[(position + i) % key.len()];
        }
        
        writer.write_all(&buffer[..bytes_read])?;
        position += bytes_read;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_symmetry() {
        let original = b"Hello, World!";
        let key = b"secret";
        
        let encrypted = xor_transform(original, key);
        let decrypted = xor_transform(&encrypted, key);
        
        assert_eq!(original, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let content = b"Test file content for encryption";
        let key = b"mykey123";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), content)?;
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            key,
        )?;
        
        xor_decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(content, decrypted_content.as_slice());
        
        Ok(())
    }
}