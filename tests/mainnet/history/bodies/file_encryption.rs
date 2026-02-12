use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    algorithm: CipherAlgorithm,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(b"unique nonce");
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let ciphertext = cipher.encrypt(&nonce, plaintext)?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    nonce: &[u8],
    algorithm: CipherAlgorithm,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(nonce);
            let plaintext = cipher.decrypt(nonce, ciphertext)?;
            Ok(plaintext)
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = ChaChaNonce::from_slice(nonce);
            let plaintext = cipher.decrypt(nonce, ciphertext)?;
            Ok(plaintext)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let plaintext = b"secret message";
        let result = encrypt_data(plaintext, CipherAlgorithm::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(
            &result.ciphertext,
            &result.nonce,
            CipherAlgorithm::Aes256Gcm,
        )
        .unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let plaintext = b"another secret";
        let result = encrypt_data(plaintext, CipherAlgorithm::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(
            &result.ciphertext,
            &result.nonce,
            CipherAlgorithm::ChaCha20Poly1305,
        )
        .unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}