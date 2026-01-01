use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordConfig {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }
}

fn generate_password(config: &PasswordConfig) -> String {
    let mut charset = String::new();
    
    if config.use_uppercase {
        charset.push_str(UPPERCASE);
    }
    if config.use_lowercase {
        charset.push_str(LOWERCASE);
    }
    if config.use_digits {
        charset.push_str(DIGITS);
    }
    if config.use_special {
        charset.push_str(SPECIAL);
    }
    
    if charset.is_empty() {
        return String::from("Error: No character set selected");
    }
    
    let mut rng = rand::thread_rng();
    let password: String = (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset.chars().nth(idx).unwrap()
        })
        .collect();
    
    password
}

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

fn parse_bool_input(input: &str) -> bool {
    match input.to_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => true,
        _ => false,
    }
}

fn main() {
    println!("Secure Password Generator");
    println!("==========================");
    
    let length_input = get_user_input(&format!("Password length (default: {}): ", DEFAULT_LENGTH));
    let length = if length_input.is_empty() {
        DEFAULT_LENGTH
    } else {
        length_input.parse().unwrap_or(DEFAULT_LENGTH)
    };
    
    let config = PasswordConfig {
        length,
        use_uppercase: parse_bool_input(&get_user_input("Include uppercase letters? (Y/n): ")),
        use_lowercase: parse_bool_input(&get_user_input("Include lowercase letters? (Y/n): ")),
        use_digits: parse_bool_input(&get_user_input("Include digits? (Y/n): ")),
        use_special: parse_bool_input(&get_user_input("Include special characters? (Y/n): ")),
    };
    
    let password = generate_password(&config);
    println!("\nGenerated Password: {}", password);
    println!("Password Length: {}", password.len());
    
    if config.use_uppercase && password.chars().any(|c| c.is_uppercase()) {
        println!("✓ Contains uppercase letters");
    }
    if config.use_lowercase && password.chars().any(|c| c.is_lowercase()) {
        println!("✓ Contains lowercase letters");
    }
    if config.use_digits && password.chars().any(|c| c.is_digit(10)) {
        println!("✓ Contains digits");
    }
    if config.use_special && password.chars().any(|c| !c.is_alphanumeric()) {
        println!("✓ Contains special characters");
    }
}