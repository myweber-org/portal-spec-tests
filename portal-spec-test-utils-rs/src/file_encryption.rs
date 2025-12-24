
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let input_data = fs::read(input_path)?;

    let encrypted_data: Vec<u8> = input_data
        .into_iter()
        .map(|byte| byte ^ encryption_key)
        .collect();

    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

pub fn process_file_interactive() -> io::Result<()> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();

    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();

    println!("Enter encryption key (leave empty for default 0xAA):");
    let mut key_input = String::new();
    io::stdin().read_line(&mut key_input)?;
    let key_input = key_input.trim();

    let key = if key_input.is_empty() {
        None
    } else {
        match u8::from_str_radix(key_input, 16) {
            Ok(k) => Some(k),
            Err(_) => {
                eprintln!("Invalid hex key, using default.");
                None
            }
        }
    };

    println!("Encrypt (e) or Decrypt (d)?");
    let mut mode = String::new();
    io::stdin().read_line(&mut mode)?;

    match mode.trim() {
        "e" => xor_encrypt_file(input_path, output_path, key),
        "d" => xor_decrypt_file(input_path, output_path, key),
        _ => {
            eprintln!("Invalid mode selected.");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption() {
        let input_data = b"Hello, World!";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(input_data).unwrap();

        let output_file = NamedTempFile::new().unwrap();

        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0xCC),
        )
        .unwrap();

        let encrypted_data = fs::read(output_file.path()).unwrap();
        assert_ne!(encrypted_data, input_data);

        let decrypted_file = NamedTempFile::new().unwrap();
        xor_decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0xCC),
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, input_data);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer)?;

        let processed_data: Vec<u8> = buffer
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect();

        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&processed_data)?;

        Ok(())
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = "secret_key";
        let cipher = XorCipher::new(key);

        let original_content = b"Test data for encryption";
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(original_content).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        cipher
            .encrypt_file(source_file.path(), encrypted_file.path())
            .unwrap();
        cipher
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_generate_random_key() {
        let key_length = 32;
        let key = generate_random_key(key_length);
        assert_eq!(key.len(), key_length);
        assert!(key.iter().any(|&b| b != 0));
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = key.to_vec();
    result.extend_from_slice(nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < 48 {
        return Err("Invalid ciphertext length".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&ciphertext[..32]);
    let nonce = Nonce::from_slice(&ciphertext[32..44]);
    let encrypted_data = &ciphertext[44..];
    
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original = b"Secret message for encryption test";
        let encrypted = encrypt_data(original).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_tampered_data() {
        let original = b"Another secret payload";
        let mut encrypted = encrypt_data(original).unwrap();
        
        encrypted[50] ^= 0xFF;
        
        let result = decrypt_data(&encrypted);
        assert!(result.is_err());
    }
}use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

pub fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt_decrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_symmetry() {
        let data = b"Hello, World!";
        let key = b"secret";
        
        let encrypted = xor_encrypt_decrypt(data, key);
        let decrypted = xor_encrypt_decrypt(&encrypted, key);
        
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_processing() -> io::Result<()> {
        let test_data = b"Test file content";
        let key = b"testkey";
        
        fs::write("test_input.txt", test_data)?;
        
        process_file("test_input.txt", "test_output.txt", key)?;
        process_file("test_output.txt", "test_restored.txt", key)?;
        
        let restored = fs::read("test_restored.txt")?;
        
        assert_eq!(test_data.to_vec(), restored);
        
        fs::remove_file("test_input.txt")?;
        fs::remove_file("test_output.txt")?;
        fs::remove_file("test_restored.txt")?;
        
        Ok(())
    }
}