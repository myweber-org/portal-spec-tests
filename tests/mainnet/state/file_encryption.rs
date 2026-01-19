use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_data = key.to_vec();
    output_data.extend_from_slice(nonce);
    output_data.extend_from_slice(&ciphertext);

    fs::write(output_path, output_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    if data.len() < 48 {
        return Err("Invalid encrypted file format".into());
    }

    let (key_bytes, rest) = data.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(12);

    let key = key_bytes.try_into()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::fs;
use std::io::{self, Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const KEY_SIZE: usize = 32;
const IV_SIZE: usize = 16;

fn generate_key() -> [u8; KEY_SIZE] {
    let mut key = [0u8; KEY_SIZE];
    rand::thread_rng().fill(&mut key);
    key
}

fn generate_iv() -> [u8; IV_SIZE] {
    let mut iv = [0u8; IV_SIZE];
    rand::thread_rng().fill(&mut iv);
    iv
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<(String, String)> {
    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let key = generate_key();
    let iv = generate_iv();

    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

    Ok((hex::encode(key), hex::encode(iv)))
}

pub fn decrypt_file(input_path: &str, output_path: &str, key_hex: &str, iv_hex: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut ciphertext = Vec::new();
    input_file.read_to_end(&mut ciphertext)?;

    let key = hex::decode(key_hex).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let iv = hex::decode(iv_hex).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if key.len() != KEY_SIZE || iv.len() != IV_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid key or IV length",
        ));
    }

    let key_array: [u8; KEY_SIZE] = key.try_into().unwrap();
    let iv_array: [u8; IV_SIZE] = iv.try_into().unwrap();

    let plaintext = Aes256CbcDec::new(&key_array.into(), &iv_array.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}