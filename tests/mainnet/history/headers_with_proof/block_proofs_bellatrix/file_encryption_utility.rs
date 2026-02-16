
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }
}

pub fn generate_key_file(path: &Path) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let mut file = fs::File::create(path)?;
    file.write_all(&key)?;
    Ok(())
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        SaltString
    },
    Argon2, PasswordHasher
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
    nonce: [u8; NONCE_SIZE]
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_material = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(&key_material.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        
        Ok(Self {
            cipher,
            nonce
        })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let nonce = Nonce::from_slice(&self.nonce);
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&self.nonce)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
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
    fn test_encryption_decryption() {
        let password = "secure_password_123!";
        let encryptor = FileEncryptor::new(password).unwrap();
        
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
    
    #[test]
    fn test_key_generation() {
        let key1 = generate_secure_key();
        let key2 = generate_secure_key();
        
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
        assert_ne!(key1, key2);
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
    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce_obj = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce_obj, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        ciphertext,
        nonce,
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
    let nonce_obj = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce_obj, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&plaintext)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn interactive_encrypt() -> Result<(), String> {
    println!("Enter input file path:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    let input_path = Path::new(input.trim());

    println!("Enter output file path:");
    let mut output = String::new();
    io::stdin()
        .read_line(&mut output)
        .map_err(|e| e.to_string())?;
    let output_path = Path::new(output.trim());

    println!("Enter encryption password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| e.to_string())?;
    let password = password.trim();

    let result = encrypt_file(input_path, output_path, password)?;
    println!("Encryption successful!");
    println!("Salt (hex): {}", hex::encode(result.salt));
    println!("Nonce (hex): {}", hex::encode(result.nonce));
    println!("Keep salt and nonce secure for decryption.");

    Ok(())
}

pub fn interactive_decrypt() -> Result<(), String> {
    println!("Enter encrypted file path:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    let input_path = Path::new(input.trim());

    println!("Enter output file path:");
    let mut output = String::new();
    io::stdin()
        .read_line(&mut output)
        .map_err(|e| e.to_string())?;
    let output_path = Path::new(output.trim());

    println!("Enter decryption password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| e.to_string())?;
    let password = password.trim();

    println!("Enter salt (hex):");
    let mut salt_hex = String::new();
    io::stdin()
        .read_line(&mut salt_hex)
        .map_err(|e| e.to_string())?;
    let salt = hex::decode(salt_hex.trim())
        .map_err(|e| e.to_string())?
        .try_into()
        .map_err(|_| "Invalid salt length")?;

    println!("Enter nonce (hex):");
    let mut nonce_hex = String::new();
    io::stdin()
        .read_line(&mut nonce_hex)
        .map_err(|e| e.to_string())?;
    let nonce = hex::decode(nonce_hex.trim())
        .map_err(|e| e.to_string())?
        .try_into()
        .map_err(|_| "Invalid nonce length")?;

    decrypt_file(input_path, output_path, password, &nonce, &salt)?;
    println!("Decryption successful!");

    Ok(())
}