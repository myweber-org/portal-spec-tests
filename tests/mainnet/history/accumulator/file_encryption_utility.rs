
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, Params
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let params = Params::new(65536, 2, 1, Some(32))
        .map_err(|e| format!("Failed to create Argon2 params: {}", e))?;
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );

    let salt_str = SaltString::encode_b64(salt)
        .map_err(|e| format!("Failed to encode salt: {}", e))?;
    
    let hash = argon2.hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    
    let key_bytes = hash.hash.ok_or("No hash generated")?.as_bytes();
    if key_bytes.len() != 32 {
        return Err("Invalid key length".to_string());
    }
    
    Ok(*Key::<Aes256Gcm>::from_slice(key_bytes))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?
        .read_to_end(&mut file_data)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
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
    let mut ciphertext = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?
        .read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read ciphertext: {}", e))?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?
        .write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption and decryption data";
        let password = "secure_password_123";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let result = encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password
        ).unwrap();
        
        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &result.nonce,
            &result.salt
        ).unwrap();
        
        assert_eq!(decrypted, test_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        PasswordHasher, SaltString, PasswordHash, PasswordVerifier
    },
    Pbkdf2
};
use rand_core::RngCore;
use std::fs;
use std::io::{self, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_LENGTH],
    pub salt: [u8; SALT_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::b64_encode(salt)
        .map_err(|e| format!("Salt encoding failed: {}", e))?;
    
    let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Derived key too short".to_string());
    }
    
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&hash_bytes[..32]);
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file_data(
    plaintext: &[u8],
    password: &str
) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file_data(
    encrypted_data: &EncryptionResult,
    password: &str
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted_data.salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted_data.nonce);
    
    cipher.decrypt(nonce, encrypted_data.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<(), String> {
    let file_data = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let encrypted = encrypt_file_data(&file_data, password)?;
    
    let mut output_data = Vec::new();
    output_data.extend_from_slice(&encrypted.salt);
    output_data.extend_from_slice(&encrypted.nonce);
    output_data.extend_from_slice(&encrypted.ciphertext);
    
    fs::write(output_path, &output_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<(), String> {
    let encrypted_data = fs::read(input_path)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    if encrypted_data.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err("Invalid encrypted file format".to_string());
    }
    
    let salt = &encrypted_data[..SALT_LENGTH];
    let nonce = &encrypted_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + NONCE_LENGTH..];
    
    let encrypted_result = EncryptionResult {
        ciphertext: ciphertext.to_vec(),
        nonce: nonce.try_into().map_err(|_| "Invalid nonce length")?,
        salt: salt.try_into().map_err(|_| "Invalid salt length")?,
    };
    
    let decrypted_data = decrypt_file_data(&encrypted_result, password)?;
    
    fs::write(output_path, &decrypted_data)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption_cycle() {
        let test_data = b"Test encryption and decryption data";
        let password = "secure_password_123";
        
        let encrypted = encrypt_file_data(test_data, password).unwrap();
        let decrypted = decrypt_file_data(&encrypted, password).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_operations() {
        let original_content = b"File content for encryption test";
        let password = "test_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let test_data = b"Sensitive information";
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_file_data(test_data, correct_password).unwrap();
        
        let result = decrypt_file_data(&encrypted, wrong_password);
        assert!(result.is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        ciphertext,
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
) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&plaintext)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data for encryption test";
        let password = "strong_password_123";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();

        let output_encrypted = NamedTempFile::new().unwrap();
        let output_decrypted = NamedTempFile::new().unwrap();

        let result = encrypt_file(
            input_file.path(),
            output_encrypted.path(),
            password,
        ).unwrap();

        decrypt_file(
            output_encrypted.path(),
            output_decrypted.path(),
            password,
            &result.nonce,
            &result.salt,
        ).unwrap();

        let mut decrypted_content = Vec::new();
        fs::File::open(output_decrypted.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();

        assert_eq!(decrypted_content, plaintext);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in &mut buffer {
        *byte ^= encryption_key;
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World!";
        let key = Some(0xAA);

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_data).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data, decrypted_data.as_slice());
    }

    #[test]
    fn test_default_key() {
        let test_data = b"Test data for default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None,
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None,
        ).unwrap();

        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data, result.as_slice());
    }
}