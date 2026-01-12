
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

pub fn process_file_interactive() -> io::Result<()> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();

    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();

    println!("Enter encryption key:");
    let mut key_input = String::new();
    io::stdin().read_line(&mut key_input)?;
    let key = key_input.trim().as_bytes();

    println!("Encrypt (e) or Decrypt (d)?");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    match choice.trim() {
        "e" => xor_encrypt_file(input_path, output_path, key),
        "d" => xor_decrypt_file(input_path, output_path, key),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid choice. Use 'e' for encrypt or 'd' for decrypt."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption_decryption() {
        let original_data = b"Test data for encryption";
        let key = b"secret_key";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        let mut input = File::create(input_file.path()).unwrap();
        input.write_all(original_data).unwrap();

        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();

        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

pub struct FileCrypto;

impl FileCrypto {
    pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let key = Self::derive_key(password, &salt);

        let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&salt)
            .and_then(|_| output_file.write_all(&iv))
            .and_then(|_| output_file.write_all(&ciphertext))
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err("File too short to contain salt and IV".to_string());
        }

        let salt = &encrypted_data[..SALT_LEN];
        let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];

        let key = Self::derive_key(password, salt);

        let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::write(output_path, plaintext)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;

        Ok(())
    }

    fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        FileCrypto::encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            "strong_password"
        ).unwrap();

        FileCrypto::decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            "strong_password"
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        FileCrypto::encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            "correct_password"
        ).unwrap();

        let result = FileCrypto::decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            "wrong_password"
        );

        assert!(result.is_err());
    }
}