
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, Box<dyn std::error::Error>> {
    let salt_str = SaltString::encode_b64(salt)?;
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_str)?;
    
    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32].try_into()?;
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let mut rng = OsRng;
    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce);
    
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_obj = Nonce::from_slice(&nonce);
    let encrypted_data = cipher.encrypt(nonce_obj, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let result = EncryptionResult {
        encrypted_data: encrypted_data.clone(),
        nonce,
        salt,
    };

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&salt)?;
    output_file.write_all(&nonce)?;
    output_file.write_all(&encrypted_data)?;

    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut encrypted_content = Vec::new();
    file.read_to_end(&mut encrypted_content)?;

    if encrypted_content.len() < SALT_SIZE + NONCE_SIZE {
        return Err("Invalid encrypted file format".into());
    }

    let salt = &encrypted_content[..SALT_SIZE];
    let nonce = &encrypted_content[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &encrypted_content[SALT_SIZE + NONCE_SIZE..];

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_obj = Nonce::from_slice(nonce);
    let decrypted_data = cipher.decrypt(nonce_obj, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;

    Ok(decrypted_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption and decryption data";
        let password = "secure_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        let encrypt_result = encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password,
        ).unwrap();

        assert_eq!(encrypt_result.nonce.len(), NONCE_SIZE);
        assert_eq!(encrypt_result.salt.len(), SALT_SIZE);

        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
        ).unwrap();

        assert_eq!(decrypted, test_data);
    }

    #[test]
    fn test_wrong_password() {
        let test_data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password,
        ).unwrap();

        let result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
        );

        assert!(result.is_err());
    }
}