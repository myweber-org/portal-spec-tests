use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

pub fn process_file() -> io::Result<()> {
    let key = b"secret_key";
    let original = "test_data.txt";
    let encrypted = "encrypted.bin";
    let decrypted = "decrypted.txt";

    if !Path::new(original).exists() {
        let mut file = fs::File::create(original)?;
        file.write_all(b"Sample data for encryption test\nSecond line of content")?;
    }

    xor_encrypt_file(original, encrypted, key)?;
    xor_decrypt_file(encrypted, decrypted, key)?;

    let original_content = fs::read_to_string(original)?;
    let decrypted_content = fs::read_to_string(decrypted)?;

    assert_eq!(original_content, decrypted_content);
    println!("Encryption and decryption completed successfully");
    
    fs::remove_file(encrypted)?;
    fs::remove_file(decrypted)?;
    
    Ok(())
}

fn main() {
    if let Err(e) = process_file() {
        eprintln!("Error: {}", e);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-key-12345";

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<&[u8]>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    process_file(input_path, output_path, key)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<&[u8]>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    process_file(input_path, output_path, key)
}

fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_path = Path::new(input_path);
    let output_path = Path::new(output_path);

    if !input_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Input file not found: {}", input_path.display()),
        ));
    }

    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut buffer = [0u8; 4096];
    let mut key_index = 0;

    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        for i in 0..bytes_read {
            buffer[i] ^= key[key_index];
            key_index = (key_index + 1) % key.len();
        }

        output_file.write_all(&buffer[..bytes_read])?;
    }

    output_file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, World! This is a test message.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(b"my-key"),
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(b"my-key"),
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }

    #[test]
    fn test_default_key() {
        let content = b"Test with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), content).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None,
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None,
        ).unwrap();

        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(content, result.as_slice());
    }
}