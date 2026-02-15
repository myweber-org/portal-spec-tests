
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
}use rand::Rng;
use std::io;

pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        PasswordGenerator {
            length,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
        }

        if !self.use_uppercase && !self.use_lowercase && !self.use_digits && !self.use_special {
            return Err("At least one character set must be enabled");
        }

        let mut character_pool = Vec::new();
        
        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
        }
        
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
        }
        
        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
        }
        
        if self.use_special {
            character_pool.extend(b'!'..=b'/');
            character_pool.extend(b':'..=b'@');
            character_pool.extend(b'['..=b'`');
            character_pool.extend(b'{'..=b'~');
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx] as char
            })
            .collect();

        Ok(password)
    }
}

fn main() {
    println!("Secure Password Generator");
    println!("=========================");
    
    let mut input = String::new();
    
    println!("Enter password length (default: 16): ");
    io::stdin().read_line(&mut input).unwrap();
    let length: usize = input.trim().parse().unwrap_or(16);
    
    input.clear();
    println!("Include uppercase letters? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    let uppercase = !input.trim().eq_ignore_ascii_case("n");
    
    input.clear();
    println!("Include lowercase letters? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    let lowercase = !input.trim().eq_ignore_ascii_case("n");
    
    input.clear();
    println!("Include digits? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    let digits = !input.trim().eq_ignore_ascii_case("n");
    
    input.clear();
    println!("Include special characters? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    let special = !input.trim().eq_ignore_ascii_case("n");
    
    let generator = PasswordGenerator::new(length)
        .uppercase(uppercase)
        .lowercase(lowercase)
        .digits(digits)
        .special(special);
    
    match generator.generate() {
        Ok(password) => {
            println!("\nGenerated Password: {}", password);
            println!("Password Length: {}", password.len());
        }
        Err(e) => println!("Error: {}", e),
    }
}