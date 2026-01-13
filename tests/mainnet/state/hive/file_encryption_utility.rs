use std::fs;
use std::io::{self, Read, Write};

const XOR_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8]) {
    for byte in data.iter_mut() {
        *byte ^= XOR_KEY;
    }
}

fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn write_file(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    base64::encode(data)
}

fn base64_decode(encoded: &str) -> io::Result<Vec<u8>> {
    base64::decode(encoded).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut data = read_file(input_path)?;
    xor_cipher(&mut data);
    let encoded = base64_encode(&data);
    write_file(output_path, encoded.as_bytes())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let encoded_data = read_file(input_path)?;
    let encoded_str = String::from_utf8_lossy(&encoded_data);
    let mut decoded_data = base64_decode(&encoded_str)?;
    xor_cipher(&mut decoded_data);
    write_file(output_path, &decoded_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Hello, Rust Encryption!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encrypt_file(input_file.path().to_str().unwrap(), 
                    encrypted_file.path().to_str().unwrap()).unwrap();
        decrypt_file(encrypted_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap()).unwrap();

        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), result);
    }
}