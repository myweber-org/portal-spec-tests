use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> io::Result<()> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&generate_nonce());

    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(nonce.as_slice())?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> io::Result<()> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }

    let (nonce_slice, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_slice);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = [0x42; 32];
        let plaintext = b"Secret data that needs protection";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = Self::generate_salt();
        let key = Self::derive_key(password, &salt)?;
        let cipher = Aes256Gcm::new(&key);
        Ok(FileEncryptor { cipher })
    }

    fn generate_salt() -> [u8; SALT_LENGTH] {
        let mut salt = [0u8; SALT_LENGTH];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    fn derive_key(password: &[u8], salt: &[u8]) -> Result<Key<Aes256Gcm>, Box<dyn std::error::Error>> {
        let mut key = [0u8; 32];
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: key.len(),
        };
        
        pbkdf2_hmac::<Sha256>(password, salt, params.rounds, &mut key)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        Ok(Key::<Aes256Gcm>::from_slice(&key).clone())
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_content = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_content)?;

        let nonce = Nonce::from_slice(&Self::generate_nonce());
        let ciphertext = self.cipher
            .encrypt(nonce, file_content.as_ref())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(nonce.as_slice())?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".into());
        }

        let (nonce_slice, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_slice);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        fs::write(output_path, plaintext)?;
        Ok(())
    }

    fn generate_nonce() -> [u8; NONCE_LENGTH] {
        let mut nonce = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }
}

pub fn process_encryption(password: &str, input_file: &str, output_file: &str, encrypt: bool) -> Result<(), Box<dyn std::error::Error>> {
    let encryptor = FileEncryptor::new(password.as_bytes())?;
    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    if encrypt {
        encryptor.encrypt_file(input_path, output_path)?;
        println!("File encrypted successfully: {}", output_file);
    } else {
        encryptor.decrypt_file(input_path, output_path)?;
        println!("File decrypted successfully: {}", output_file);
    }

    Ok(())
}