
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0xAA;

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
    
    if args.len() < 3 {
        eprintln!("Usage: {} <input_file> <output_file> [key]", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    let key = if args.len() > 3 {
        args[3].parse().unwrap_or(DEFAULT_KEY)
    } else {
        DEFAULT_KEY
    };
    
    match process_file(input_path, output_path, key) {
        Ok(()) => println!("File processed successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xCC;
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_processing() -> io::Result<()> {
        let test_data = b"Test data for encryption";
        let input_path = "test_input.tmp";
        let output_path = "test_output.tmp";
        
        fs::write(input_path, test_data)?;
        process_file(input_path, output_path, DEFAULT_KEY)?;
        
        let processed = fs::read(output_path)?;
        assert_ne!(processed, test_data);
        
        process_file(output_path, input_path, DEFAULT_KEY)?;
        let restored = fs::read(input_path)?;
        assert_eq!(restored, test_data);
        
        fs::remove_file(input_path)?;
        fs::remove_file(output_path)?;
        
        Ok(())
    }
}