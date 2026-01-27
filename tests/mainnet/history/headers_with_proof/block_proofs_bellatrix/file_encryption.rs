
use std::env;
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

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <input_file> <output_file> [key]", args[0]);
        eprintln!("Example: {} secret.txt encrypted.txt 0xAA", args[0]);
        std::process::exit(1);
    }
    
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    
    let key = if args.len() > 3 {
        u8::from_str_radix(args[3].trim_start_matches("0x"), 16)
            .unwrap_or_else(|_| {
                eprintln!("Invalid key format. Using default key: 0x{:X}", DEFAULT_KEY);
                DEFAULT_KEY
            })
    } else {
        DEFAULT_KEY
    };
    
    match process_file(input_path, output_path, key) {
        Ok(_) => println!("File processed successfully with key 0x{:02X}", key),
        Err(e) => eprintln!("Error processing file: {}", e),
    }
}