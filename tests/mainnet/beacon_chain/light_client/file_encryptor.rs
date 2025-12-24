use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let mut buffer = [0u8; 1024];
    let key_len = key.len();
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for i in 0..bytes_read {
            buffer[i] ^= key[i % key_len];
        }
        
        output_file.write_all(&buffer[..bytes_read])?;
    }
    
    Ok(())
}

pub fn xor_decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, World! This is a test file.";
        let key = b"secret_key";
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), original_content).unwrap();
        
        xor_encrypt_file(input_temp.path(), encrypted_temp.path(), key).unwrap();
        xor_decrypt_file(encrypted_temp.path(), decrypted_temp.path(), key).unwrap();
        
        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}