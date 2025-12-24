use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, DEFAULT_KEY);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        let original = data.clone();
        
        xor_cipher(&mut data, DEFAULT_KEY);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, DEFAULT_KEY);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let restore_file = NamedTempFile::new()?;
        
        let test_data = b"Hello, XOR encryption!";
        fs::write(input_file.path(), test_data)?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        )?;
        
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, test_data);
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            restore_file.path().to_str().unwrap()
        )?;
        
        let restored = fs::read(restore_file.path())?;
        assert_eq!(restored, test_data);
        
        Ok(())
    }
}