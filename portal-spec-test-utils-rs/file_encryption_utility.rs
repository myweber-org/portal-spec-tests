use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    aes_cipher: Option<Aes256Gcm>,
    chacha_cipher: Option<ChaCha20Poly1305>,
}

impl FileEncryptor {
    pub fn new_aes(key: &[u8; 32]) -> Result<Self, EncryptionError> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        Ok(FileEncryptor {
            aes_cipher: Some(cipher),
            chacha_cipher: None,
        })
    }

    pub fn new_chacha(key: &[u8; 32]) -> Result<Self, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        Ok(FileEncryptor {
            aes_cipher: None,
            chacha_cipher: Some(cipher),
        })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let (ciphertext, nonce) = match (&self.aes_cipher, &self.chacha_cipher) {
            (Some(cipher), _) => {
                let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
                let ciphertext = cipher
                    .encrypt(&nonce, plaintext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                (ciphertext, nonce.to_vec())
            }
            (_, Some(cipher)) => {
                let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
                let ciphertext = cipher
                    .encrypt(&nonce, plaintext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                (ciphertext, nonce.to_vec())
            }
            _ => return Err(EncryptionError::CryptoError("No cipher initialized".to_string())),
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted file".to_string()));
        }

        let (nonce, ciphertext) = data.split_at(12);

        let plaintext = match (&self.aes_cipher, &self.chacha_cipher) {
            (Some(cipher), _) => {
                let nonce = Nonce::from_slice(nonce);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
            (_, Some(cipher)) => {
                let nonce = chacha20poly1305::Nonce::from_slice(nonce);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
            _ => return Err(EncryptionError::CryptoError("No cipher initialized".to_string())),
        };

        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let key = [0x42; 32];
        let encryptor = FileEncryptor::new_aes(&key).unwrap();

        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "Test data for AES encryption").unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_content = fs::read_to_string(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, "Test data for AES encryption\n");
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let key = [0x24; 32];
        let encryptor = FileEncryptor::new_chacha(&key).unwrap();

        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "Test data for ChaCha encryption").unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_content = fs::read_to_string(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, "Test data for ChaCha encryption\n");
    }
}