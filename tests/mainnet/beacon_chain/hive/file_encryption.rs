
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let key_bytes = key.as_bytes();
    if key_bytes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Encryption key cannot be empty",
        ));
    }

    let mut content = fs::read(input_path)?;
    xor_cipher(&mut content, key_bytes);
    fs::write(output_path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret";
        let mut data = b"hello world".to_vec();
        let original = data.clone();

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() {
        let key = "test_key";
        let mut input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        write!(input_file, "Sample file content").unwrap();

        process_file(input_file.path(), output_file.path(), key).unwrap();

        let encrypted = fs::read(output_file.path()).unwrap();
        assert_ne!(encrypted, b"Sample file content");

        let mut temp_file = NamedTempFile::new().unwrap();
        process_file(output_file.path(), temp_file.path(), key).unwrap();
        let decrypted = fs::read(temp_file.path()).unwrap();
        assert_eq!(decrypted, b"Sample file content");
    }
}
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    let path = Path::new(input_path);
    if !path.exists() {
        return Err("Input file does not exist".to_string());
    }

    let mut input_file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&encrypted_data)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = b"secret_key";
        let original_content = b"Hello, this is a test message for XOR encryption!";

        let input_temp_file = NamedTempFile::new().unwrap();
        let encrypted_temp_file = NamedTempFile::new().unwrap();
        let decrypted_temp_file = NamedTempFile::new().unwrap();

        fs::write(input_temp_file.path(), original_content).unwrap();

        xor_encrypt_file(
            input_temp_file.path().to_str().unwrap(),
            encrypted_temp_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        xor_decrypt_file(
            encrypted_temp_file.path().to_str().unwrap(),
            decrypted_temp_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let decrypted_content = fs::read(decrypted_temp_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    
    let input_file = &args[1];
    let output_file = &args[2];
    
    match process_file(input_file, output_file, DEFAULT_KEY) {
        Ok(_) => println!("File processed successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        let original = data.clone();
        let key = 0x55;
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_processing() -> io::Result<()> {
        let test_data = b"Hello, World!";
        let input_path = "test_input.txt";
        let output_path = "test_output.txt";
        
        fs::write(input_path, test_data)?;
        
        process_file(input_path, output_path, DEFAULT_KEY)?;
        
        let processed = fs::read(output_path)?;
        assert_ne!(processed, test_data);
        
        process_file(output_path, "test_restored.txt", DEFAULT_KEY)?;
        let restored = fs::read("test_restored.txt")?;
        assert_eq!(restored, test_data);
        
        fs::remove_file(input_path)?;
        fs::remove_file(output_path)?;
        fs::remove_file("test_restored.txt")?;
        
        Ok(())
    }
}