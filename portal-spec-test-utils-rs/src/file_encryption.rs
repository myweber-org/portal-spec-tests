
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptedData {
    pub salt: [u8; SALT_LEN],
    pub iv: [u8; IV_LEN],
    pub ciphertext: Vec<u8>,
}

pub fn encrypt_file_data(plaintext: &[u8], password: &str) -> Result<EncryptedData, Box<dyn Error>> {
    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);
    
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);
    
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    
    Ok(EncryptedData {
        salt,
        iv,
        ciphertext,
    })
}

pub fn decrypt_file_data(encrypted: &EncryptedData, password: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &encrypted.salt, PBKDF2_ITERATIONS, &mut key);
    
    let plaintext = Aes256CbcDec::new(&key.into(), &encrypted.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted.ciphertext)?;
    
    Ok(plaintext)
}

pub fn encrypt_to_file(data: &[u8], password: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let encrypted = encrypt_file_data(data, password)?;
    let encoded = bincode::serialize(&encrypted)?;
    std::fs::write(output_path, encoded)?;
    Ok(())
}

pub fn decrypt_from_file(password: &str, input_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let encoded = std::fs::read(input_path)?;
    let encrypted: EncryptedData = bincode::deserialize(&encoded)?;
    decrypt_file_data(&encrypted, password)
}