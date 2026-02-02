
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    xor_cipher(&mut content, key);
    fs::write(output_path, content)
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        xor_cipher(&mut data, 0xAA);
        assert_eq!(data, vec![0xAA, 0x55, 0x00, 0xFF]);
        xor_cipher(&mut data, 0xAA);
        assert_eq!(data, vec![0x00, 0xFF, 0xAA, 0x55]);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        write!(input_file, "Test data 123!")?;
        let input_path = input_file.path();

        let output_file = NamedTempFile::new()?;
        let output_path = output_file.path();

        encrypt_file(input_path, output_path, Some(0x77))?;
        let encrypted = fs::read(output_path)?;
        assert_ne!(encrypted, b"Test data 123!");

        let decrypted_file = NamedTempFile::new()?;
        let decrypted_path = decrypted_file.path();
        decrypt_file(output_path, decrypted_path, Some(0x77))?;
        let decrypted = fs::read(decrypted_path)?;
        assert_eq!(decrypted, b"Test data 123!");

        Ok(())
    }
}