
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(&key)?;
    output.write_all(nonce)?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if content.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short",
        ));
    }

    let (key_bytes, rest) = content.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);

    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}
use std::fs;
use std::io::{self, Read, Write};
use base64::{Engine as _, engine::general_purpose};

fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn encrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let encrypted = xor_cipher(&buffer, key.as_bytes());
    let encoded = general_purpose::STANDARD.encode(encrypted);
    
    fs::write(output_path, encoded)?;
    Ok(())
}

fn decrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let encoded = fs::read_to_string(input_path)?;
    let encrypted = general_purpose::STANDARD.decode(encoded.trim())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    let decrypted = xor_cipher(&encrypted, key.as_bytes());
    
    fs::write(output_path, decrypted)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <encrypt|decrypt> <input> <output> <key>", args[0]);
        std::process::exit(1);
    }
    
    let operation = &args[1];
    let input = &args[2];
    let output = &args[3];
    let key = &args[4];
    
    match operation.as_str() {
        "encrypt" => encrypt_file(input, output, key),
        "decrypt" => decrypt_file(input, output, key),
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let plaintext = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_data = Vec::with_capacity(key.len() + ciphertext.len());
    output_data.extend_from_slice(&key);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain valid encrypted data",
        ));
    }
    
    let (key_bytes, ciphertext) = encrypted_data.split_at(32);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let decrypted_file = NamedTempFile::new().unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use std::fs;
use std::io::{self, Read, Write};

fn xor_encrypt_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt_decrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input_file> <output_file> <key>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let key = args[3].as_bytes();

    process_file(input_path, output_path, key)?;
    println!("File processed successfully: {} -> {}", input_path, output_path);

    Ok(())
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub struct FileCrypto;

impl FileCrypto {
    pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut file_data = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
        
        let mut salt = [0u8; SALT_LENGTH];
        let mut iv = [0u8; IV_LENGTH];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);
        
        let mut key = [0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, KEY_ITERATIONS, &mut key);
        
        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let encrypted_data = cipher.encrypt_padded_vec_mut::<Pkcs7>(&mut file_data);
        
        let mut output = Vec::with_capacity(SALT_LENGTH + IV_LENGTH + encrypted_data.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&iv);
        output.extend_from_slice(&encrypted_data);
        
        fs::write(output_path, &output).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
    
    pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let encrypted_data = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
        
        if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let salt = &encrypted_data[0..SALT_LENGTH];
        let iv = &encrypted_data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH];
        let ciphertext = &encrypted_data[SALT_LENGTH + IV_LENGTH..];
        
        let mut key = [0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
        
        let cipher = Aes256CbcDec::new(&key.into(), iv.into());
        let decrypted_data = cipher.decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, &decrypted_data).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
    
    pub fn generate_key_file(output_path: &str, password: &str) -> Result<(), String> {
        let mut salt = [0u8; SALT_LENGTH];
        rand::thread_rng().fill_bytes(&mut salt);
        
        let mut key = [0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, KEY_ITERATIONS, &mut key);
        
        let mut output = Vec::with_capacity(SALT_LENGTH + KEY_LENGTH);
        output.extend_from_slice(&salt);
        output.extend_from_slice(&key);
        
        fs::write(output_path, &output).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(&key)?;
    output.write_all(nonce)?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let data = fs::read(input_path)?;
    if data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key and nonce",
        ));
    }

    let (key_bytes, rest) = data.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);

    let key = key_bytes.try_into().unwrap();
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();
        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), result);
    }
}