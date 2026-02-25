
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

#[derive(Debug)]
pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: CipherAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: CipherAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::generate(&mut OsRng);
                let ciphertext = cipher.encrypt(&nonce, plaintext)?;
                let mut result = nonce.to_vec();
                result.extend(ciphertext);
                Ok(result)
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
                let ciphertext = cipher.encrypt(&nonce, plaintext)?;
                let mut result = nonce.to_vec();
                result.extend(ciphertext);
                Ok(result)
            }
        }
    }

    pub fn decrypt(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                if ciphertext.len() < 12 {
                    return Err("Invalid ciphertext length".into());
                }
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
                let nonce = Nonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, encrypted_data).map_err(|e| e.into())
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                if ciphertext.len() < 12 {
                    return Err("Invalid ciphertext length".into());
                }
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
                let nonce = ChaChaNonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, encrypted_data).map_err(|e| e.into())
            }
        }
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        let plaintext = b"Secret message for AES encryption";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::ChaCha20Poly1305);
        let key = generate_random_key();
        let plaintext = b"Secret message for ChaCha encryption";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        let wrong_key = generate_random_key();
        let plaintext = b"Test message";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let result = encryptor.decrypt(&ciphertext, &wrong_key);
        
        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if content.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short",
        ));
    }

    let (key_bytes, ciphertext) = content.split_at(32);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::{Rng, thread_rng};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

const KEY_SIZE: usize = 32;
const IV_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; KEY_SIZE],
}

impl FileEncryptor {
    pub fn new() -> Self {
        let mut key = [0u8; KEY_SIZE];
        thread_rng().fill(&mut key);
        FileEncryptor { key }
    }

    pub fn with_key(key: [u8; KEY_SIZE]) -> Self {
        FileEncryptor { key }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let mut iv = [0u8; IV_SIZE];
        thread_rng().fill(&mut iv);

        let cipher = Aes256Cbc::new_from_slices(&self.key, &iv)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;

        let ciphertext = cipher.encrypt_vec(&plaintext);

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&iv)
            .map_err(|e| format!("Failed to write IV: {}", e))?;
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        if buffer.len() < IV_SIZE {
            return Err("File too short to contain IV".to_string());
        }

        let iv = &buffer[..IV_SIZE];
        let ciphertext = &buffer[IV_SIZE..];

        let cipher = Aes256Cbc::new_from_slices(&self.key, iv)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;

        let plaintext = cipher.decrypt_vec(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;

        Ok(())
    }

    pub fn get_key(&self) -> &[u8; KEY_SIZE] {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}