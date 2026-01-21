use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&generate_nonce());

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output = File::create(output_path)?;
    output.write_all(nonce.as_slice())?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < NONCE_SIZE {
        return Err("File too short to contain nonce".into());
    }

    let (nonce_slice, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_slice);
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output = File::create(output_path)?;
    output.write_all(&plaintext)?;

    Ok(())
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let bytes = password.as_bytes();
    for (i, &byte) in bytes.iter().cycle().take(32).enumerate() {
        key[i] = byte;
    }
    *Key::<Aes256Gcm>::from_slice(&key)
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::fs;
use std::io::{self, Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);
    key
}

fn generate_iv() -> [u8; 16] {
    let mut iv = [0u8; 16];
    rand::thread_rng().fill(&mut iv);
    iv
}

fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let key = generate_key();
    let iv = generate_iv();

    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&iv)?;
    output_file.write_all(&ciphertext)?;

    println!("Encryption complete.");
    println!("Key (hex): {}", hex::encode(key));
    println!("IV (hex): {}", hex::encode(iv));
    println!("Output saved to: {}", output_path);

    Ok(())
}

fn decrypt_file(input_path: &str, output_path: &str, key_hex: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 16 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain IV",
        ));
    }

    let iv = &encrypted_data[0..16];
    let ciphertext = &encrypted_data[16..];

    let key = hex::decode(key_hex).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if key.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Key must be 32 bytes (256 bits)",
        ));
    }

    let decrypted_data = Aes256CbcDec::new(key.as_slice().into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;

    println!("Decryption complete.");
    println!("Output saved to: {}", output_path);

    Ok(())
}

fn main() -> io::Result<()> {
    println!("File Encryption Utility");
    println!("1. Encrypt file");
    println!("2. Decrypt file");

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    match choice {
        "1" => {
            println!("Enter input file path:");
            let mut input_path = String::new();
            io::stdin().read_line(&mut input_path)?;
            let input_path = input_path.trim();

            println!("Enter output file path:");
            let mut output_path = String::new();
            io::stdin().read_line(&mut output_path)?;
            let output_path = output_path.trim();

            encrypt_file(input_path, output_path)
        }
        "2" => {
            println!("Enter input file path:");
            let mut input_path = String::new();
            io::stdin().read_line(&mut input_path)?;
            let input_path = input_path.trim();

            println!("Enter output file path:");
            let mut output_path = String::new();
            io::stdin().read_line(&mut output_path)?;
            let output_path = output_path.trim();

            println!("Enter encryption key (hex):");
            let mut key_hex = String::new();
            io::stdin().read_line(&mut key_hex)?;
            let key_hex = key_hex.trim();

            decrypt_file(input_path, output_path, key_hex)
        }
        _ => {
            println!("Invalid choice");
            Ok(())
        }
    }
}