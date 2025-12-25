use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, ParamsBuilder,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::new_with_secret(
        &[],
        ParamsBuilder::new()
            .m_cost(65536)
            .t_cost(3)
            .p_cost(4)
            .build()
            .map_err(|e| format!("Argon2 params error: {}", e))?,
    )
    .map_err(|e| format!("Argon2 initialization error: {}", e))?;

    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| format!("Salt encoding error: {}", e))?;

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("Key derivation error: {}", e))?;

    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes[..32]);
    Ok(Key::<Aes256Gcm>::from_slice(&key).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;

    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;

    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted_data = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    output_file
        .write_all(&encrypted_data)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

    Ok(EncryptionResult {
        encrypted_data,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> Result<Vec<u8>, String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;

    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let decrypted_data = cipher
        .decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    output_file
        .write_all(&decrypted_data)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(decrypted_data)
}

pub fn generate_random_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Test data for encryption and decryption";
        let password = "secure_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let encryption_result = encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");

        let decrypted_data = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &encryption_result.nonce,
            &encryption_result.salt,
        )
        .expect("Decryption should succeed");

        assert_eq!(decrypted_data, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Sensitive data";
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let encryption_result = encrypt_file(input_file.path(), encrypted_file.path(), correct_password)
            .expect("Encryption should succeed");

        let result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
            &encryption_result.nonce,
            &encryption_result.salt,
        );

        assert!(result.is_err());
    }
}