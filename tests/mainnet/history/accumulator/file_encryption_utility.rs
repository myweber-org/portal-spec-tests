
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(Self { key })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&rand::random::<[u8; NONCE_SIZE]>());
        
        let encrypted_data = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&encrypted_data)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        
        let decrypted_data = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::File::create(output_path)?.write_all(&decrypted_data)?;
        
        Ok(())
    }
}

pub fn generate_secure_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
    InvalidKeyLength,
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn generate_key(&self) -> Result<Vec<u8>, EncryptionError> {
        let key_length = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => 32,
            EncryptionAlgorithm::ChaCha20Poly1305 => 32,
        };

        let mut key = vec![0u8; key_length];
        OsRng.fill_bytes(&mut key);
        Ok(key)
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut file_data)?;

        let (ciphertext, nonce) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce_bytes = self.generate_nonce(12);
                let nonce = Nonce::from_slice(&nonce_bytes);
                let ciphertext = cipher
                    .encrypt(nonce, file_data.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                (ciphertext, nonce_bytes)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce_bytes = self.generate_nonce(12);
                let nonce = ChaChaNonce::from_slice(&nonce_bytes);
                let ciphertext = cipher
                    .encrypt(nonce, file_data.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                (ciphertext, nonce_bytes)
            }
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < 12 {
            return Err(EncryptionError::CryptoError(
                "Invalid encrypted data length".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::from_slice(nonce_bytes);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce = ChaChaNonce::from_slice(nonce_bytes);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn generate_nonce(&self, size: usize) -> Vec<u8> {
        let mut nonce = vec![0u8; size];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = encryptor.generate_key().unwrap();

        let test_data = b"Test encryption data";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path(), &key)
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path(), &key)
            .unwrap();

        let mut decrypted_data = Vec::new();
        let mut file = fs::File::open(decrypted_file.path()).unwrap();
        file.read_to_end(&mut decrypted_data).unwrap();

        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = encryptor.generate_key().unwrap();

        let test_data = b"Test ChaCha20-Poly1305 encryption";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path(), &key)
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path(), &key)
            .unwrap();

        let mut decrypted_data = Vec::new();
        let mut file = fs::File::open(decrypted_file.path()).unwrap();
        file.read_to_end(&mut decrypted_data).unwrap();

        assert_eq!(decrypted_data, test_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::error::Error;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn encrypt_data(
    plaintext: &[u8],
    password: &str,
) -> Result<EncryptionResult, Box<dyn Error>> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        &salt,
        PBKDF2_ITERATIONS,
        &mut key,
        Params::default(),
    )?;

    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce,
    })
}

pub fn decrypt_data(
    encrypted: &EncryptionResult,
    password: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        &encrypted.salt,
        PBKDF2_ITERATIONS,
        &mut key,
        Params::default(),
    )?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&encrypted.nonce), encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret message for encryption";
        let password = "StrongPassword123!";

        let encrypted = encrypt_data(plaintext, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "CorrectPassword";
        let wrong_password = "WrongPassword";

        let encrypted = encrypt_data(plaintext, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);

        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let salt_str = SaltString::encode_b64(salt)?;
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_str)?;
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
    Ok(key)
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);

    let key_bytes = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted_data = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

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
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;

    let key_bytes = derive_key(password, salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    let decrypted_data = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;

    Ok(())
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

        let result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &result.nonce,
            &result.salt,
        )
        .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_content);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Secret data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let decrypt_result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
            &result.nonce,
            &result.salt,
        );

        assert!(decrypt_result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(nonce.as_slice())?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn process_encryption() -> Result<(), Box<dyn std::error::Error>> {
    let password = "secure_password_123";
    let encryptor = FileEncryptor::new(password)?;
    
    let test_data = b"Sample confidential data for encryption test";
    let input_path = Path::new("test_input.txt");
    let encrypted_path = Path::new("test_encrypted.bin");
    let decrypted_path = Path::new("test_decrypted.txt");
    
    fs::write(input_path, test_data)?;
    
    encryptor.encrypt_file(input_path, encrypted_path)?;
    println!("File encrypted successfully");
    
    encryptor.decrypt_file(encrypted_path, decrypted_path)?;
    println!("File decrypted successfully");
    
    let decrypted_data = fs::read(decrypted_path)?;
    assert_eq!(test_data.to_vec(), decrypted_data);
    
    fs::remove_file(input_path)?;
    fs::remove_file(encrypted_path)?;
    fs::remove_file(decrypted_path)?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    process_encryption()
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    Argon2, 
    password_hash::{PasswordHasher, SaltString}
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_material = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key_bytes = key_material.as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;
        
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output = File::create(output_path)?;
        output.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn secure_delete_file(path: &Path) -> Result<(), std::io::Error> {
    let file_size = fs::metadata(path)?.len();
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    
    for _ in 0..3 {
        let random_data: Vec<u8> = (0..file_size).map(|_| rand::random()).collect();
        file.write_all(&random_data)?;
        file.flush()?;
        file.seek(std::io::SeekFrom::Start(0))?;
    }
    
    fs::remove_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123!";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        let original_content = b"Secret data that needs protection";
        let mut temp_input = NamedTempFile::new().unwrap();
        temp_input.write_all(original_content).unwrap();
        
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(temp_input.path(), temp_encrypted.path()).unwrap();
        encryptor.decrypt_file(temp_encrypted.path(), temp_decrypted.path()).unwrap();
        
        let mut decrypted_content = Vec::new();
        File::open(temp_decrypted.path()).unwrap()
            .read_to_end(&mut decrypted_content).unwrap();
        
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}