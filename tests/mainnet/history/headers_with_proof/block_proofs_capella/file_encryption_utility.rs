
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    key: &[u8; 32],
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes: [u8; 12] = OsRng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    key: &[u8; 32],
    nonce: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(plaintext)
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn save_key_to_file(key: &[u8; 32], path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let hex_key = hex::encode(key);
    fs::write(path, hex_key)?;
    Ok(())
}

pub fn load_key_from_file(path: &Path) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let hex_key = fs::read_to_string(path)?;
    let mut key = [0u8; 32];
    hex::decode_to_slice(hex_key.trim(), &mut key)?;
    Ok(key)
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)?;
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)?;
        let key_bytes = password_hash.hash.ok_or("Failed to derive key")?.as_bytes();
        
        if key_bytes.len() != 32 {
            return Err("Key length mismatch".into());
        }
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_content = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_content)?;
        
        let mut rng = OsRng;
        let nonce_bytes: [u8; NONCE_SIZE] = rng.random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, file_content.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        output_data.extend_from_slice(&nonce_bytes);
        output_data.extend_from_slice(&ciphertext);
        
        fs::File::create(output_path)?.write_all(&output_data)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".into());
        }
        
        let nonce_bytes = &encrypted_data[..NONCE_SIZE];
        let ciphertext = &encrypted_data[NONCE_SIZE..];
        
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::File::create(output_path)?.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();
        
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        rand_core::RngCore,
        PasswordHasher, SaltString
    },
    Pbkdf2
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

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
        .map_err(|e| format!("Failed to encode salt: {}", e))?;
    
    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("PBKDF2 failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short for key")?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<EncryptionResult, String> {
    let plaintext = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
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
    
    fs::write(output_path, &ciphertext)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt: &[u8],
    nonce: &[u8]
) -> Result<Vec<u8>, String> {
    let ciphertext = fs::read(input_path)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, &plaintext)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
    
    Ok(plaintext)
}

pub fn save_metadata(metadata_path: &Path, salt: &[u8], nonce: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(metadata_path)?;
    file.write_all(b"SALT:")?;
    file.write_all(salt)?;
    file.write_all(b"\nNONCE:")?;
    file.write_all(nonce)?;
    Ok(())
}

pub fn load_metadata(metadata_path: &Path) -> Result<(Vec<u8>, Vec<u8>), String> {
    let content = fs::read_to_string(metadata_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 2 {
        return Err("Invalid metadata format".to_string());
    }
    
    let salt_line = lines[0];
    let nonce_line = lines[1];
    
    if !salt_line.starts_with("SALT:") || !nonce_line.starts_with("NONCE:") {
        return Err("Metadata missing required headers".to_string());
    }
    
    let salt_hex = &salt_line[5..];
    let nonce_hex = &nonce_line[6..];
    
    let salt = hex::decode(salt_hex)
        .map_err(|e| format!("Invalid salt hex: {}", e))?;
    let nonce = hex::decode(nonce_hex)
        .map_err(|e| format!("Invalid nonce hex: {}", e))?;
    
    Ok((salt, nonce))
}