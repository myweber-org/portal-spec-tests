
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub struct FileCrypto;

impl FileCrypto {
    pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut rng = rand::thread_rng();
        
        let mut salt = [0u8; SALT_LENGTH];
        rng.fill_bytes(&mut salt);
        
        let mut iv = [0u8; IV_LENGTH];
        rng.fill_bytes(&mut iv);
        
        let mut key = [0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, KEY_ITERATIONS, &mut key);
        
        let plaintext = fs::read(input_path).map_err(|e| e.to_string())?;
        
        let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);
        
        let mut output = Vec::with_capacity(SALT_LENGTH + IV_LENGTH + ciphertext.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&iv);
        output.extend_from_slice(&ciphertext);
        
        fs::write(output_path, &output).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let data = fs::read(input_path).map_err(|e| e.to_string())?;
        
        if data.len() < SALT_LENGTH + IV_LENGTH {
            return Err("File too short".to_string());
        }
        
        let salt = &data[0..SALT_LENGTH];
        let iv = &data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH];
        let ciphertext = &data[SALT_LENGTH + IV_LENGTH..];
        
        let mut key = [0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
        
        let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| e.to_string())?;
        
        fs::write(output_path, &plaintext).map_err(|e| e.to_string())?;
        
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
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        let password = "strong_password_123";
        
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
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_wrong_password() {
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