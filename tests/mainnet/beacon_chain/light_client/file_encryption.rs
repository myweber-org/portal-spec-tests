
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
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
        eprintln!("Usage: {} <input> <output> <key>", args[0]);
        eprintln!("Example: {} secret.txt encrypted.txt 42", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    let key_arg = &args[3];
    
    let key: u8 = match key_arg.parse() {
        Ok(k) => k,
        Err(_) => {
            eprintln!("Error: Key must be a number between 0 and 255");
            std::process::exit(1);
        }
    };
    
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }
    
    match process_file(input_path, output_path, key) {
        Ok(_) => println!("File processed successfully: {} -> {}", input_path, output_path),
        Err(e) => {
            eprintln!("Error processing file: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0x55;
        
        xor_cipher(&mut data, key);
        xor_cipher(&mut data, key);
        
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let test_data = b"Hello, World!";
        let input_path = "test_input.txt";
        let output_path = "test_output.txt";
        let key = 42;
        
        fs::write(input_path, test_data)?;
        process_file(input_path, output_path, key)?;
        process_file(output_path, "test_decrypted.txt", key)?;
        
        let decrypted = fs::read("test_decrypted.txt")?;
        assert_eq!(decrypted, test_data);
        
        fs::remove_file(input_path)?;
        fs::remove_file(output_path)?;
        fs::remove_file("test_decrypted.txt")?;
        
        Ok(())
    }
}