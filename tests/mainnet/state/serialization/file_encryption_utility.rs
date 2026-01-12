use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-encryption-key-2024";

pub struct FileCipher {
    key: Vec<u8>,
}

impl FileCipher {
    pub fn new(key: Option<&[u8]>) -> Self {
        let key = key.unwrap_or(DEFAULT_KEY).to_vec();
        FileCipher { key }
    }

    pub fn encrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &str, dest_path: &str, is_encrypt: bool) -> io::Result<()> {
        let source = Path::new(source_path);
        let dest = Path::new(dest_path);

        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source file not found: {}", source_path),
            ));
        }

        let mut source_file = fs::File::open(source)?;
        let mut dest_file = fs::File::create(dest)?;

        let mut buffer = [0u8; 4096];
        let key_len = self.key.len();
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();

            for byte in processed_buffer.iter_mut() {
                *byte ^= self.key[key_index];
                key_index = (key_index + 1) % key_len;
            }

            dest_file.write_all(&processed_buffer)?;
        }

        if is_encrypt {
            println!("File encrypted successfully: {} -> {}", source_path, dest_path);
        } else {
            println!("File decrypted successfully: {} -> {}", source_path, dest_path);
        }

        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        self.process_text(text.as_bytes())
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted = self.process_text(data);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_text(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        let mut result = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            result.push(byte ^ self.key[i % key_len]);
        }

        result
    }
}

pub fn validate_key(key: &[u8]) -> bool {
    !key.is_empty() && key.len() >= 8
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_string() {
        let cipher = FileCipher::new(Some(b"test-key-123"));
        let original = "Hello, World! This is a secret message.";
        
        let encrypted = cipher.encrypt_string(original);
        let decrypted = cipher.decrypt_string(&encrypted);
        
        assert_eq!(original, decrypted);
        assert_ne!(original.as_bytes(), encrypted);
    }

    #[test]
    fn test_file_operations() {
        let cipher = FileCipher::new(Some(b"file-test-key"));
        let original_content = b"Test file content for encryption demonstration.";
        
        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(
            source_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        cipher.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key(b"valid-key-123"));
        assert!(validate_key(b"12345678"));
        assert!(!validate_key(b"short"));
        assert!(!validate_key(b""));
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
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, argon2::password_hash::Error> {
    let salt_str = SaltString::encode_b64(salt)?;
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_str)?;
    
    let hash_bytes = password_hash.hash.unwrap().as_bytes();
    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> io::Result<EncryptionResult> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

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
) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;

    let key = derive_key(password, salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

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
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand::RngCore;
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

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], argon2::Error> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map(|_| key)
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_data)?;

    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);

    let key_bytes = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let result = EncryptionResult {
        ciphertext: ciphertext.clone(),
        nonce: nonce_bytes,
        salt,
    };

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&salt)?;
    output_file.write_all(&nonce_bytes)?;
    output_file.write_all(&ciphertext)?;

    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err("Invalid encrypted file format".into());
    }

    let (salt, rest) = encrypted_data.split_at(SALT_SIZE);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);

    let key_bytes = derive_key(password, salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::File::create(output_path)?.write_all(&plaintext)?;
    Ok(plaintext)
}

pub fn verify_password(
    encrypted_path: &Path,
    password: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut file_data = Vec::new();
    fs::File::open(encrypted_path)?.read_to_end(&mut file_data)?;

    if file_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err("Invalid encrypted file format".into());
    }

    let (salt, _) = file_data.split_at(SALT_SIZE);
    let key_bytes = derive_key(password, salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let _cipher = Aes256Gcm::new(key);

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data for encryption test";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        let decrypted = decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
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
        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let result = decrypt_file(encrypted_file.path(), decrypted_file.path(), wrong_password);
        assert!(result.is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs::{self, File};
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
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
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
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&ciphertext).map_err(|e| e.to_string())?;

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
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&plaintext).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn generate_random_key() -> Result<[u8; 32], String> {
    let mut rng = OsRng;
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Test secret data";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        fs::write(temp_input.path(), plaintext).unwrap();

        let password = "strong_password_123";
        let result = encrypt_file(temp_input.path(), temp_encrypted.path(), password).unwrap();

        decrypt_file(
            temp_encrypted.path(),
            temp_decrypted.path(),
            password,
            &result.nonce,
            &result.salt,
        )
        .unwrap();

        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        fs::write(temp_input.path(), plaintext).unwrap();

        let password = "correct_password";
        let result = encrypt_file(temp_input.path(), temp_encrypted.path(), password).unwrap();

        let wrong_password = "wrong_password";
        let decrypt_result = decrypt_file(
            temp_encrypted.path(),
            temp_decrypted.path(),
            wrong_password,
            &result.nonce,
            &result.salt,
        );

        assert!(decrypt_result.is_err());
    }
}
use aes_gcm::{
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
        let nonce = Nonce::from_slice(&Self::generate_nonce());

        let encrypted_data = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(nonce)?;
        output_file.write_all(&encrypted_data)?;

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

    fn generate_nonce() -> [u8; NONCE_SIZE] {
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }
}

pub fn generate_key_file(password: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let encryptor = FileEncryptor::new(password)?;
    let mut key_file = fs::File::create(output_path)?;
    key_file.write_all(&encryptor.key)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();

        let original_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_data).unwrap();

        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}