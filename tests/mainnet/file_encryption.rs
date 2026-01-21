
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data to encrypt";
        let key = b"mykey123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
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
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

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

        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

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
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err("File too short to contain salt and IV".to_string());
        }

        let (salt, rest) = encrypted_data.split_at(SALT_LEN);
        let (iv, ciphertext) = rest.split_at(IV_LEN);

        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);

        let decrypted = Aes256CbcDec::new(&key.into(), iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&decrypted)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        FileCrypto::encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();

        FileCrypto::decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        FileCrypto::encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();

        let result = FileCrypto::decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            wrong_password
        );

        assert!(result.is_err());
    }
}