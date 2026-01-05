
use std::fs::{File, read, write};
use std::io::{Read, Write};
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

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &str, output_path: &str, is_encrypt: bool) -> Result<(), String> {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(format!("Input file does not exist: {}", input_path));
        }

        let mut input_file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data = self.xor_transform(&buffer);

        let mut output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        self.xor_transform(text.as_bytes())
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted_bytes = self.xor_transform(data);
        String::from_utf8_lossy(&decrypted_bytes).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let cipher = XorCipher::new("secret_key");
        let original = "Hello, World!";
        
        let encrypted = cipher.encrypt_string(original);
        let decrypted = cipher.decrypt_string(&encrypted);
        
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let cipher = XorCipher::new("test_key");
        let original_content = b"Sample file content for encryption test";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        cipher.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_empty_key() {
        let cipher = XorCipher::new("");
        let data = b"test data";
        let encrypted = cipher.xor_transform(data);
        assert_eq!(data, &encrypted[..]);
    }
}use std::fs;
use std::io::{self, Read, Write};

fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

fn process_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_cipher(&mut buffer, key.as_bytes());

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input> <output> <key>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let key = &args[3];

    process_file(input_path, output_path, key)?;
    println!("File processed successfully");

    Ok(())
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_file(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(b"unique nonce");
    let ciphertext = cipher.encrypt(nonce, data)?;
    Ok(ciphertext)
}

pub fn decrypt_file(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(b"unique nonce");
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0u8; 32];
        let data = b"secret message";
        
        let encrypted = encrypt_file(data, &key).unwrap();
        let decrypted = decrypt_file(&encrypted, &key).unwrap();
        
        assert_eq!(data, decrypted.as_slice());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonOsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

pub fn derive_key(password: &str, salt: &SaltString) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    let hash_bytes = password_hash
        .hash
        .ok_or("No hash generated")?
        .as_bytes()
        .to_vec();

    if hash_bytes.len() < 32 {
        return Err("Derived key too short".to_string());
    }

    let key_bytes: [u8; 32] = hash_bytes[..32]
        .try_into()
        .map_err(|_| "Key conversion failed".to_string())?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let salt = SaltString::generate(&mut ArgonOsRng);
    let key = derive_key(password, &salt)?;

    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut combined_data = salt.as_bytes().to_vec();
    combined_data.extend_from_slice(data);

    let ciphertext = cipher
        .encrypt(nonce, combined_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_data(
    ciphertext: &[u8],
    nonce: &[u8; NONCE_SIZE],
    password: &str,
) -> Result<Vec<u8>, String> {
    if ciphertext.len() < 22 {
        return Err("Ciphertext too short".to_string());
    }

    let salt = SaltString::from_b64(&base64::encode(&ciphertext[..22]))
        .map_err(|e| format!("Invalid salt: {}", e))?;

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, &ciphertext[22..])
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(plaintext[22..].to_vec())
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let result = encrypt_data(&buffer, password)?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    output_file
        .write_all(&result.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;

    output_file
        .write_all(&result.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if buffer.len() < NONCE_SIZE {
        return Err("File too short to contain nonce".to_string());
    }

    let nonce_bytes: [u8; NONCE_SIZE] = buffer[..NONCE_SIZE]
        .try_into()
        .map_err(|_| "Invalid nonce size".to_string())?;

    let ciphertext = &buffer[NONCE_SIZE..];
    let plaintext = decrypt_data(ciphertext, &nonce_bytes, password)?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    output_file
        .write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let data = b"Secret data for encryption test";
        let password = "strong_password_123";

        let encrypted = encrypt_data(data, password).unwrap();
        let decrypted = decrypt_data(&encrypted.ciphertext, &encrypted.nonce, password).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_operations() {
        let test_data = b"Test file content for encryption";
        let password = "file_password_456";

        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encrypt_file(input_file.path(), output_file.path(), password).unwrap();
        decrypt_file(output_file.path(), decrypted_file.path(), password).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }

    #[test]
    fn test_wrong_password() {
        let data = b"Sensitive information";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let encrypted = encrypt_data(data, password).unwrap();
        let result = decrypt_data(&encrypted.ciphertext, &encrypted.nonce, wrong_password);

        assert!(result.is_err());
    }
}