
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path
};
use anyhow::{Context, Result};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
    nonce: Nonce,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(15000, 2, 1, Some(32)).unwrap()
        );
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .context("Failed to hash password")?;
        
        let key_bytes = password_hash.hash.unwrap().as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        Ok(Self { cipher, nonce })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let mut input_file = File::open(input_path)
            .context("Failed to open input file")?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)
            .context("Failed to read input file")?;
        
        let ciphertext = self.cipher
            .encrypt(self.nonce, plaintext.as_ref())
            .context("Encryption failed")?;
        
        let mut output_file = File::create(output_path)
            .context("Failed to create output file")?;
        output_file.write_all(&ciphertext)
            .context("Failed to write encrypted data")?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let mut input_file = File::open(input_path)
            .context("Failed to open encrypted file")?;
        let mut ciphertext = Vec::new();
        input_file.read_to_end(&mut ciphertext)
            .context("Failed to read encrypted file")?;
        
        let plaintext = self.cipher
            .decrypt(self.nonce, ciphertext.as_ref())
            .context("Decryption failed - possibly wrong password")?;
        
        let mut output_file = File::create(output_path)
            .context("Failed to create output file")?;
        output_file.write_all(&plaintext)
            .context("Failed to write decrypted data")?;
        
        Ok(())
    }
}

pub fn generate_secure_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() -> Result<()> {
        let password = "secure_password_123!";
        let encryptor = FileEncryptor::new(password)?;
        
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), original_content)?;
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())?;
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content.as_slice(), decrypted_content.as_slice());
        
        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Key derivation failed");
    key
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;

    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret data to protect";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let result = encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
            &result.nonce,
            &result.salt,
        )
        .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_content);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            self.xor_transform(&mut processed_buffer, &mut key_index);

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    fn xor_transform(&self, data: &mut [u8], key_index: &mut usize) {
        for byte in data.iter_mut() {
            *byte ^= self.key[*key_index];
            *key_index = (*key_index + 1) % self.key.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("test_key_123");
        let test_data = b"Hello, this is a secret message!";
        
        let mut encrypted = test_data.to_vec();
        let mut key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        assert_eq!(encrypted, test_data);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = XORCipher::new("secure_password");
        
        let original_content = b"Confidential data that needs protection";
        let mut source_file = NamedTempFile::new()?;
        source_file.write_all(original_content)?;
        
        let encrypted_file = NamedTempFile::new()?;
        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;
        
        let decrypted_file = NamedTempFile::new()?;
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path())?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;
        
        assert_eq!(decrypted_content, original_content);
        Ok(())
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
    rng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
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
) -> Result<Vec<u8>, String> {
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
    output_file.write_all(&plaintext).map_err(|e| e.to_string())?;

    Ok(plaintext)
}

pub fn interactive_encrypt() -> Result<(), String> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin()
        .read_line(&mut input_path)
        .map_err(|e| e.to_string())?;
    let input_path = input_path.trim();

    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin()
        .read_line(&mut output_path)
        .map_err(|e| e.to_string())?;
    let output_path = output_path.trim();

    println!("Enter encryption password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| e.to_string())?;
    let password = password.trim();

    let result = encrypt_file(Path::new(input_path), Path::new(output_path), password)?;
    println!("Encryption successful!");
    println!("Nonce (hex): {}", hex::encode(result.nonce));
    println!("Salt (hex): {}", hex::encode(result.salt));
    println!("Save these values for decryption.");

    Ok(())
}

pub fn interactive_decrypt() -> Result<(), String> {
    println!("Enter encrypted file path:");
    let mut input_path = String::new();
    io::stdin()
        .read_line(&mut input_path)
        .map_err(|e| e.to_string())?;
    let input_path = input_path.trim();

    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin()
        .read_line(&mut output_path)
        .map_err(|e| e.to_string())?;
    let output_path = output_path.trim();

    println!("Enter decryption password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| e.to_string())?;
    let password = password.trim();

    println!("Enter nonce (hex):");
    let mut nonce_hex = String::new();
    io::stdin()
        .read_line(&mut nonce_hex)
        .map_err(|e| e.to_string())?;
    let nonce_hex = nonce_hex.trim();
    let nonce = hex::decode(nonce_hex)
        .map_err(|e| e.to_string())?
        .try_into()
        .map_err(|_| "Invalid nonce length")?;

    println!("Enter salt (hex):");
    let mut salt_hex = String::new();
    io::stdin()
        .read_line(&mut salt_hex)
        .map_err(|e| e.to_string())?;
    let salt_hex = salt_hex.trim();
    let salt = hex::decode(salt_hex)
        .map_err(|e| e.to_string())?
        .try_into()
        .map_err(|_| "Invalid salt length")?;

    decrypt_file(
        Path::new(input_path),
        Path::new(output_path),
        password,
        &nonce,
        &salt,
    )?;

    println!("Decryption successful!");
    Ok(())
}