
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0xAA, 0xBB, 0xCC, 0xDD];
        let key = vec![0x12, 0x34];
        
        xor_cipher(&mut data, &key);
        assert_eq!(data, vec![0xB8, 0x8F, 0xDE, 0xE9]);
        
        xor_cipher(&mut data, &key);
        assert_eq!(data, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        let test_data = b"Secret message for encryption test";
        input_file.write_all(test_data)?;
        
        let output_file = NamedTempFile::new()?;
        let key = b"encryption_key";
        
        process_file(input_file.path(), output_file.path(), key)?;
        
        let mut encrypted = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut encrypted)?;
        
        assert_ne!(encrypted.as_slice(), test_data);
        
        let mut decrypted = encrypted.clone();
        xor_cipher(&mut decrypted, key);
        
        assert_eq!(decrypted.as_slice(), test_data);
        
        Ok(())
    }
}