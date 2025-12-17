use std::fs::File;
use std::io::{Read, Result};
use sha2::{Sha256, Digest};

pub fn compute_file_hash(file_path: &str) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_file_hash(file_path: &str, expected_hash: &str) -> Result<bool> {
    let computed_hash = compute_file_hash(file_path)?;
    Ok(computed_hash == expected_hash.to_lowercase())
}