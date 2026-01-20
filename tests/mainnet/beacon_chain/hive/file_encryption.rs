
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut output_file = fs::File::create(output_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = input_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % self.key.len();
            }

            output_file.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = "secret_key";
        let cipher = XorCipher::new(key);

        let original_content = b"Hello, this is a test message for encryption!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_empty_file() {
        let key = "test_key";
        let cipher = XorCipher::new(key);

        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        cipher.encrypt_file(input_file.path(), output_file.path()).unwrap();
        
        let output_content = fs::read(output_file.path()).unwrap();
        assert!(output_content.is_empty());
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
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, String> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| e.to_string())?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| e.to_string())?;
        
        let mut output = File::create(output_path).map_err(|e| e.to_string())?;
        output.write_all(nonce.as_slice()).map_err(|e| e.to_string())?;
        output.write_all(&ciphertext).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data).map_err(|e| e.to_string())?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;
        
        let mut output = File::create(output_path).map_err(|e| e.to_string())?;
        output.write_all(&plaintext).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn generate_salt() -> Vec<u8> {
        generate_random_bytes(SALT_SIZE)
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn encrypt_directory(
    encryptor: &FileEncryptor,
    dir_path: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    }
    
    for entry in fs::read_dir(dir_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_file() {
            let output_path = output_dir.join(path.file_name().unwrap());
            encryptor.encrypt_file(&path, &output_path)?;
        }
    }
    
    Ok(())
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, encrypted_data)?;
    
    let key_path = format!("{}.key", output_path);
    fs::write(key_path, key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key_data = fs::read(key_path)?;
    
    let key = key_data.as_slice().try_into()
        .map_err(|_| "Invalid key length")?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    
    Ok(())
}