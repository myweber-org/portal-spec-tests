
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_LENGTH: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub iv: [u8; IV_LENGTH],
}

pub fn encrypt_data(
    plaintext: &[u8],
    password: &str,
) -> Result<EncryptionResult, Box<dyn Error>> {
    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let mut key = [0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext);

    Ok(EncryptionResult {
        ciphertext,
        salt,
        iv,
    })
}

pub fn decrypt_data(
    encrypted: &EncryptionResult,
    password: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut key = [0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &encrypted.salt, PBKDF2_ITERATIONS, &mut key);

    let plaintext = Aes256CbcDec::new(&key.into(), &encrypted.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted.ciphertext)?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret message for encryption";
        let password = "strong_password_123";

        let encrypted = encrypt_data(plaintext, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Another secret";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let encrypted = encrypt_data(plaintext, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);

        assert!(result.is_err());
    }
}