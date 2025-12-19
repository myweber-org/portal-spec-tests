
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    pbkdf2_hmac,
    Params
};
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: key.len(),
    };
    
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key);
    *Key::<Aes256Gcm>::from_slice(&key)
}

pub fn encrypt_file(password: &str, input_path: &Path) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    
    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_file(password: &str, result: &EncryptionResult) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &result.salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&result.nonce);
    
    cipher.decrypt(nonce, result.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn save_encrypted_data(result: &EncryptionResult, output_path: &Path) -> Result<(), String> {
    let mut file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    file.write_all(&result.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    
    file.write_all(&result.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    
    file.write_all(&result.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn load_encrypted_data(input_path: &Path) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    if data.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err("File too short to contain valid encrypted data".to_string());
    }
    
    let salt = data[..SALT_LENGTH].try_into()
        .map_err(|_| "Failed to extract salt")?;
    
    let nonce = data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH].try_into()
        .map_err(|_| "Failed to extract nonce")?;
    
    let ciphertext = data[SALT_LENGTH + NONCE_LENGTH..].to_vec();
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce,
    })
}