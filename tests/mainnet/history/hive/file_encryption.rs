
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_crypt(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_crypt(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }

    process_file(input_path, output_path, DEFAULT_KEY)?;
    println!("File processed successfully with XOR key 0x{:02X}", DEFAULT_KEY);

    Ok(())
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut buffer = [0u8; 1024];
    let key_len = key.len();
    let mut key_index = 0;

    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        for byte in buffer.iter_mut().take(bytes_read) {
            *byte ^= key[key_index];
            key_index = (key_index + 1) % key_len;
        }

        output_file.write_all(&buffer[..bytes_read])?;
    }

    output_file.flush()?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption_decryption() {
        let key = b"secret_key";
        let original_data = b"Hello, this is a test message for XOR encryption!";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_data).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        xor_encrypt_file(input_file.path(), encrypted_file.path(), key).unwrap();
        xor_decrypt_file(encrypted_file.path(), decrypted_file.path(), key).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}