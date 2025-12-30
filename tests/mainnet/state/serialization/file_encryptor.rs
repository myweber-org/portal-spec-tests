
use std::fs;
use std::io::{self, Read, Write};
use base64::{Engine as _, engine::general_purpose};

const KEY: &[u8] = b"secret-encryption-key-32-bytes-long!";

fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file_content = fs::read(input_path)?;
    
    xor_cipher(&mut file_content, KEY);
    let encoded = general_purpose::STANDARD.encode(&file_content);
    
    fs::write(output_path, encoded)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let encoded = fs::read_to_string(input_path)?;
    let mut decoded = general_purpose::STANDARD.decode(encoded)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    xor_cipher(&mut decoded, KEY);
    fs::write(output_path, decoded)
}

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, encrypt: bool) -> io::Result<()> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    
    if encrypt {
        xor_cipher(&mut buffer, KEY);
        let encoded = general_purpose::STANDARD.encode(&buffer);
        writer.write_all(encoded.as_bytes())?;
    } else {
        let decoded = general_purpose::STANDARD.decode(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut decrypted = decoded;
        xor_cipher(&mut decrypted, KEY);
        writer.write_all(&decrypted)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original = b"Hello, this is a secret message!";
        let mut temp_input = NamedTempFile::new().unwrap();
        let temp_output = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        temp_input.write_all(original).unwrap();
        
        encrypt_file(temp_input.path().to_str().unwrap(), 
                    temp_output.path().to_str().unwrap()).unwrap();
        
        decrypt_file(temp_output.path().to_str().unwrap(),
                    temp_decrypted.path().to_str().unwrap()).unwrap();
        
        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original.as_slice(), decrypted_content);
    }
}