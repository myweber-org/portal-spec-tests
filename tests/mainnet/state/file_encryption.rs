
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data = xor_encrypt(&input_data, key);
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let decrypted_data = xor_decrypt(&input_data, key);
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn xor_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn process_files() -> io::Result<()> {
    let test_data = b"Secret message for encryption test";
    let key = b"encryption_key";
    
    let input_file = "test_input.txt";
    let encrypted_file = "test_encrypted.bin";
    let decrypted_file = "test_decrypted.txt";
    
    fs::write(input_file, test_data)?;
    
    xor_encrypt_file(input_file, encrypted_file, key)?;
    xor_decrypt_file(encrypted_file, decrypted_file, key)?;
    
    let decrypted_content = fs::read(decrypted_file)?;
    assert_eq!(test_data.to_vec(), decrypted_content);
    
    fs::remove_file(input_file)?;
    fs::remove_file(encrypted_file)?;
    fs::remove_file(decrypted_file)?;
    
    println!("File encryption test completed successfully");
    Ok(())
}
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &str, output_path: &str, is_encrypt: bool) -> Result<(), String> {
        let input_path_obj = Path::new(input_path);
        let output_path_obj = Path::new(output_path);

        if !input_path_obj.exists() {
            return Err(format!("Input file does not exist: {}", input_path));
        }

        let mut input_file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data = self.xor_process(&buffer);

        let mut output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        let operation = if is_encrypt { "encrypted" } else { "decrypted" };
        println!("File successfully {}: {} -> {}", operation, input_path, output_path);
        
        Ok(())
    }

    fn xor_process(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

pub fn save_key_to_file(key: &[u8], file_path: &str) -> Result<(), String> {
    let hex_string: String = key.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();

    fs::write(file_path, hex_string)
        .map_err(|e| format!("Failed to save key file: {}", e))
}

pub fn load_key_from_file(file_path: &str) -> Result<Vec<u8>, String> {
    let hex_string = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;

    if hex_string.len() % 2 != 0 {
        return Err("Invalid key file format".to_string());
    }

    (0..hex_string.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_string[i..i + 2], 16)
            .map_err(|_| "Invalid hex character in key file".to_string()))
        .collect()
}