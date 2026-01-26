use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

#[derive(Debug)]
pub enum CipherType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    key: &[u8],
    cipher_type: CipherType,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
            let nonce = Nonce::generate(&mut OsRng);
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        CipherType::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
    cipher_type: CipherType,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
            let nonce = Nonce::from_slice(nonce);
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
        CipherType::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
            let nonce = ChaChaNonce::from_slice(nonce);
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_roundtrip() {
        let key = [0x42; 32];
        let plaintext = b"Test encryption data";
        let result = encrypt_data(plaintext, &key, CipherType::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce, CipherType::Aes256Gcm).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_roundtrip() {
        let key = [0x24; 32];
        let plaintext = b"Another test message";
        let result = encrypt_data(plaintext, &key, CipherType::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce, CipherType::ChaCha20Poly1305).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}