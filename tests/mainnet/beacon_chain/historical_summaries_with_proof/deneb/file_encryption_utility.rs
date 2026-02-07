
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        rand_core::RngCore,
        PasswordHasher, SaltString
    },
    Params, Pbkdf2
};
use std::fs;
use std::io::{self, Read, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| format!("Salt encoding failed: {}", e))?;
    
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: 32,
    };
    
    let password_hash = Pbkdf2
        .hash_password_customized(
            password.as_bytes(),
            None,
            None,
            params,
            &salt_string
        )
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let key_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    Ok(*key)
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
    salt: &[u8],
    nonce: &[u8]
) -> Result<Vec<u8>, String> {
    if nonce.len() != NONCE_LENGTH {
        return Err(format!("Invalid nonce length: expected {}", NONCE_LENGTH));
    }
    
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read ciphertext: {}", e))?;
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;
    
    Ok(plaintext)
}

pub fn generate_secure_password(length: usize) -> Result<String, String> {
    if length < 12 {
        return Err("Password length must be at least 12 characters".to_string());
    }
    
    let mut rng = OsRng;
    let mut password_bytes = vec![0u8; length];
    rng.fill_bytes(&mut password_bytes);
    
    let password = base64::encode(&password_bytes)
        .chars()
        .take(length)
        .collect();
    
    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption and decryption data";
        let password = "secure_test_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        let enc_result = encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted = decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
            &enc_result.salt,
            &enc_result.nonce
        ).unwrap();
        
        assert_eq!(decrypted, test_data);
    }
    
    #[test]
    fn test_password_generation() {
        let password = generate_secure_password(16).unwrap();
        assert_eq!(password.len(), 16);
        assert!(password.chars().all(|c| c.is_ascii_graphic()));
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, Params,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> io::Result<Key<Aes256Gcm>> {
    let salt_str = SaltString::b64_encode(salt)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, Some(32))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
    );

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let key_bytes = password_hash.hash.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Failed to generate hash")
    })?;

    Ok(*Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes()))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> io::Result<EncryptionResult> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);

    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let result = EncryptionResult {
        ciphertext: ciphertext.clone(),
        salt,
        nonce,
    };

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&salt)?;
    output_file.write_all(&nonce)?;
    output_file.write_all(&ciphertext)?;

    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain salt and nonce",
        ));
    }

    let salt = &encrypted_data[0..SALT_LENGTH];
    let nonce = &encrypted_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + NONCE_LENGTH..];

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Secret data that needs protection";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let test_data = b"Secret data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();
        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let result = decrypt_file(encrypted_file.path(), decrypted_file.path(), wrong_password);
        assert!(result.is_err());
    }
}