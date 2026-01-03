
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn encrypt_string(data: &str, key: Option<u8>) -> Vec<u8> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    data.bytes()
        .map(|b| b ^ encryption_key)
        .collect()
}

pub fn decrypt_string(data: &[u8], key: Option<u8>) -> String {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    data.iter()
        .map(|&b| (b ^ encryption_key) as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption() {
        let original = "Hello, World!";
        let encrypted = encrypt_string(original, Some(0x42));
        let decrypted = decrypt_string(&encrypted, Some(0x42));
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original_content = b"Secret data";
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(original_content)?;
        let input_path = input_file.path().to_str().unwrap();

        let output_file = NamedTempFile::new()?;
        let output_path = output_file.path().to_str().unwrap();

        encrypt_file(input_path, output_path, Some(0x99))?;
        let mut encrypted_content = Vec::new();
        fs::File::open(output_path)?.read_to_end(&mut encrypted_content)?;
        assert_ne!(original_content, encrypted_content.as_slice());

        let decrypted_file = NamedTempFile::new()?;
        let decrypted_path = decrypted_file.path().to_str().unwrap();
        decrypt_file(output_path, decrypted_path, Some(0x99))?;

        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_path)?.read_to_end(&mut decrypted_content)?;
        assert_eq!(original_content, decrypted_content.as_slice());

        Ok(())
    }
}