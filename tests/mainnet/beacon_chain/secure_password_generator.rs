
use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn generate_password(length: usize, use_uppercase: bool, use_lowercase: bool, use_digits: bool, use_symbols: bool) -> String {
    let mut charset = String::new();
    
    if use_uppercase { charset.push_str(UPPERCASE); }
    if use_lowercase { charset.push_str(LOWERCASE); }
    if use_digits { charset.push_str(DIGITS); }
    if use_symbols { charset.push_str(SYMBOLS); }
    
    if charset.is_empty() {
        charset = format!("{}{}{}", UPPERCASE, LOWERCASE, DIGITS);
    }
    
    let charset_bytes: Vec<u8> = charset.bytes().collect();
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect()
}

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn parse_bool_input(input: &str) -> bool {
    input.to_lowercase() == "y" || input.to_lowercase() == "yes" || input == "1"
}

fn main() {
    println!("Secure Password Generator");
    println!("=========================");
    
    let length_input = get_user_input(&format!("Password length (default: {}):", DEFAULT_LENGTH));
    let length: usize = length_input.parse().unwrap_or(DEFAULT_LENGTH);
    
    let use_uppercase = parse_bool_input(&get_user_input("Include uppercase letters? (y/n):"));
    let use_lowercase = parse_bool_input(&get_user_input("Include lowercase letters? (y/n):"));
    let use_digits = parse_bool_input(&get_user_input("Include digits? (y/n):"));
    let use_symbols = parse_bool_input(&get_user_input("Include symbols? (y/n):"));
    
    let password = generate_password(length, use_uppercase, use_lowercase, use_digits, use_symbols);
    
    println!("\nGenerated Password: {}", password);
    println!("Password length: {} characters", password.len());
}