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
    use std::fs;

    #[test]
    fn test_xor_symmetry() {
        let original = b"Hello, World!";
        let key = 0xCC;
        
        let encrypted = xor_encrypt(original, key);
        assert_ne!(original, encrypted.as_slice());
        
        let decrypted = xor_decrypt(&encrypted, key);
        assert_eq!(original, decrypted.as_slice());
    }

    #[test]
    fn test_file_operations() -> io::Result<()> {
        let test_content = b"Test file content for encryption";
        let input_path = "test_input.txt";
        let encrypted_path = "test_encrypted.bin";
        let decrypted_path = "test_decrypted.txt";
        
        fs::write(input_path, test_content)?;
        
        encrypt_file(input_path, encrypted_path)?;
        decrypt_file(encrypted_path, decrypted_path)?;
        
        let decrypted_content = fs::read(decrypted_path)?;
        assert_eq!(test_content, decrypted_content.as_slice());
        
        fs::remove_file(input_path)?;
        fs::remove_file(encrypted_path)?;
        fs::remove_file(decrypted_path)?;
        
        Ok(())
    }
}