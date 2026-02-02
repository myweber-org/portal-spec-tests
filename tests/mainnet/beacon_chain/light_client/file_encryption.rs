use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer.iter().map(|byte| byte ^ key).collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World!";
        let key = Some(0x42);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
}
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    
    xor_cipher(&mut content, key);
    
    fs::write(output_path, &content)?;
    Ok(())
}

fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input> <output> [key]", args[0]);
        std::process::exit(1);
    }
    
    let operation = &args[1];
    let input = &args[2];
    let output = &args[3];
    let key = args.get(4).and_then(|k| k.parse::<u8>().ok());
    
    match operation.as_str() {
        "encrypt" => encrypt_file(input, output, key),
        "decrypt" => decrypt_file(input, output, key),
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xCC;
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        let test_data = b"Hello, XOR encryption!";
        input_file.write_all(test_data)?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0x7B)
        )?;
        
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, test_data);
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            input_file.path().to_str().unwrap(),
            Some(0x7B)
        )?;
        
        let decrypted = fs::read(input_file.path())?;
        assert_eq!(decrypted, test_data);
        
        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|byte| byte ^ key).collect()
}

pub fn xor_decrypt(data: &[u8], key: u8) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    process_file(input_path, output_path, DEFAULT_KEY)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    process_file(input_path, output_path, DEFAULT_KEY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_cipher() {
        let original = b"Hello, World!";
        let encrypted = xor_encrypt(original, DEFAULT_KEY);
        let decrypted = xor_decrypt(&encrypted, DEFAULT_KEY);
        
        assert_ne!(original, encrypted.as_slice());
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