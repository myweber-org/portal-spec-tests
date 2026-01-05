use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let cipher_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in &mut buffer {
        *byte ^= cipher_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data for testing!";
        let key = Some(0xAA);

        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();

        fs::write(input_temp.path(), original_content).unwrap();

        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let encrypted_data = fs::read(encrypted_temp.path()).unwrap();
        assert_ne!(encrypted_data, original_content);

        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(decrypted_data, original_content);
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
    
    let mut result = Vec::with_capacity(key.len() + nonce.len() + ciphertext.len());
    result.extend_from_slice(&key);
    result.extend_from_slice(nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(encrypted_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if encrypted_data.len() < 48 {
        return Err("Invalid encrypted data length".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&encrypted_data[0..32]);
    let nonce = Nonce::from_slice(&encrypted_data[32..44]);
    let ciphertext = &encrypted_data[44..];
    
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret message for encryption test";
        let encrypted = encrypt_data(original_data).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_tampered_data() {
        let original_data = b"Another secret message";
        let mut encrypted = encrypt_data(original_data).unwrap();
        
        encrypted[50] ^= 0xFF;
        
        let result = decrypt_data(&encrypted);
        assert!(result.is_err());
    }
}
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

pub fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_encrypt() {
        let data = b"Hello World";
        let key = b"secret";
        let encrypted = xor_encrypt(data, key);
        let decrypted = xor_encrypt(&encrypted, key);
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_empty_data() {
        let data = b"";
        let key = b"key";
        let result = xor_encrypt(data, key);
        assert!(result.is_empty());
    }
}
use std::fs;
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

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    fn process(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let cipher = XorCipher::new(key);
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let encrypted = cipher.encrypt(&buffer);
    let mut output = fs::File::create(output_path)?;
    output.write_all(&encrypted)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let cipher = XorCipher::new(key);
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let decrypted = cipher.decrypt(&buffer);
    let mut output = fs::File::create(output_path)?;
    output.write_all(&decrypted)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_cipher() {
        let cipher = XorCipher::new("secret");
        let original = b"Hello, World!";
        let encrypted = cipher.encrypt(original);
        let decrypted = cipher.decrypt(&encrypted);

        assert_eq!(original.to_vec(), decrypted);
        assert_ne!(original.to_vec(), encrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let test_data = b"Test file content for encryption";
        let input_path = Path::new("test_input.txt");
        let encrypted_path = Path::new("test_encrypted.bin");
        let decrypted_path = Path::new("test_decrypted.txt");

        fs::write(input_path, test_data)?;

        encrypt_file(input_path, encrypted_path, "mykey")?;
        decrypt_file(encrypted_path, decrypted_path, "mykey")?;

        let decrypted_content = fs::read(decrypted_path)?;
        assert_eq!(test_data.to_vec(), decrypted_content);

        fs::remove_file(input_path)?;
        fs::remove_file(encrypted_path)?;
        fs::remove_file(decrypted_path)?;

        Ok(())
    }
}