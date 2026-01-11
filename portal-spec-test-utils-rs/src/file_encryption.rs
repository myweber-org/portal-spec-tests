
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{pbkdf2_hmac, Params};
use rand::{rngs::OsRng, RngCore};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub struct EncryptionResult {
    pub salt: [u8; SALT_LENGTH],
    pub iv: [u8; IV_LENGTH],
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    let params = Params {
        rounds: KEY_ITERATIONS,
        output_length: KEY_LENGTH,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key)
        .expect("PBKDF2 should not fail");
    key
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);

    let key = derive_key(password, &salt);
    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(data);

    Ok(EncryptionResult {
        salt,
        iv,
        ciphertext,
    })
}

pub fn decrypt_data(encrypted: &EncryptionResult, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt);
    let cipher = Aes256CbcDec::new(&key.into(), &encrypted.iv.into());

    cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted.ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let encrypted = encrypt_data(&data, password)?;

    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output
        .write_all(&encrypted.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output
        .write_all(&encrypted.iv)
        .map_err(|e| format!("Failed to write IV: {}", e))?;
    output
        .write_all(&encrypted.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let data = fs::read(input_path).map_err(|e| format!("Failed to read input file: {}", e))?;

    if data.len() < SALT_LENGTH + IV_LENGTH {
        return Err("Input file too short".to_string());
    }

    let salt = &data[0..SALT_LENGTH];
    let iv = &data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH];
    let ciphertext = &data[SALT_LENGTH + IV_LENGTH..];

    let encrypted = EncryptionResult {
        salt: salt.try_into().unwrap(),
        iv: iv.try_into().unwrap(),
        ciphertext: ciphertext.to_vec(),
    };

    let decrypted = decrypt_data(&encrypted, password)?;

    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output
        .write_all(&decrypted)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let data = b"Secret message for encryption test";
        let password = "strong_password_123";

        let encrypted = encrypt_data(data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        let test_data = b"File encryption test content";
        fs::write(input_file.path(), test_data).unwrap();

        let password = "file_encryption_password";

        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();

        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data, decrypted_data.as_slice());
    }

    #[test]
    fn test_wrong_password() {
        let data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let encrypted = encrypt_data(data, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);

        assert!(result.is_err());
    }
}