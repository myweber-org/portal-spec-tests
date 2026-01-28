
use std::fs;
use std::io::{self, Read, Write};
use base64::{Engine as _, engine::general_purpose};

const CHUNK_SIZE: usize = 8192;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let key_len = key.len();
    let mut key_index = 0;
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for i in 0..bytes_read {
            buffer[i] ^= key[key_index];
            key_index = (key_index + 1) % key_len;
        }
        
        let encoded = general_purpose::STANDARD.encode(&buffer[..bytes_read]);
        writeln!(output_file, "{}", encoded)?;
    }
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let reader = io::BufReader::new(input_file);
    let key_len = key.len();
    let mut key_index = 0;
    
    for line in io::BufRead::lines(reader) {
        let line = line?;
        let decoded = general_purpose::STANDARD.decode(line)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let mut decrypted_chunk = decoded;
        for byte in decrypted_chunk.iter_mut() {
            *byte ^= key[key_index];
            key_index = (key_index + 1) % key_len;
        }
        
        output_file.write_all(&decrypted_chunk)?;
    }
    
    Ok(())
}

pub fn generate_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, this is a secret message!";
        let key = b"my-secret-key-123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}