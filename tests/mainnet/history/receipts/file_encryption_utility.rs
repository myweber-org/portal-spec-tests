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

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
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
        assert_ne!(encrypted.as_slice(), original);
        
        let decrypted = xor_decrypt(&encrypted, key);
        assert_eq!(decrypted.as_slice(), original);
    }
    
    #[test]
    fn test_file_operations() -> io::Result<()> {
        let content = b"Test file content for encryption";
        let mut temp_input = NamedTempFile::new()?;
        temp_input.write_all(content)?;
        
        let temp_output = NamedTempFile::new()?;
        
        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap()
        )?;
        
        let mut encrypted_content = Vec::new();
        fs::File::open(temp_output.path())?.read_to_end(&mut encrypted_content)?;
        assert_ne!(encrypted_content, content);
        
        let temp_decrypted = NamedTempFile::new()?;
        decrypt_file(
            temp_output.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap()
        )?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(temp_decrypted.path())?.read_to_end(&mut decrypted_content)?;
        assert_eq!(decrypted_content, content);
        
        Ok(())
    }
}