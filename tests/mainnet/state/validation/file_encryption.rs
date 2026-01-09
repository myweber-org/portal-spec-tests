
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_cipher(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let operation = &args[1];
    let input_path = Path::new(&args[2]);
    let output_path = Path::new(&args[3]);

    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }

    match operation.as_str() {
        "encrypt" | "decrypt" => {
            process_file(input_path, output_path, DEFAULT_KEY)?;
            println!("{} completed successfully", operation);
        }
        _ => {
            eprintln!("Error: First argument must be 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xAA;

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        let test_data = b"Hello, World!";
        input_file.write_all(test_data)?;

        let output_file = NamedTempFile::new()?;
        let input_path = input_file.path();
        let output_path = output_file.path();

        process_file(input_path, output_path, DEFAULT_KEY)?;

        let encrypted_data = fs::read(output_path)?;
        assert_ne!(encrypted_data, test_data);

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(&encrypted_data)?;
        
        let final_path = NamedTempFile::new()?.path().to_path_buf();
        process_file(temp_file.path(), &final_path, DEFAULT_KEY)?;

        let decrypted_data = fs::read(final_path)?;
        assert_eq!(decrypted_data, test_data);

        Ok(())
    }
}