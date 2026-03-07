use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_symbols: bool,
}

impl PasswordGenerator {
    fn new() -> Self {
        PasswordGenerator {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }

    fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    fn with_numbers(mut self, enable: bool) -> Self {
        self.use_numbers = enable;
        self
    }

    fn with_symbols(mut self, enable: bool) -> Self {
        self.use_symbols = enable;
        self
    }

    fn generate(&self) -> Result<String, String> {
        let mut character_set = String::new();
        
        if self.use_uppercase {
            character_set.push_str(UPPERCASE);
        }
        if self.use_lowercase {
            character_set.push_str(LOWERCASE);
        }
        if self.use_numbers {
            character_set.push_str(NUMBERS);
        }
        if self.use_symbols {
            character_set.push_str(SYMBOLS);
        }

        if character_set.is_empty() {
            return Err("At least one character set must be enabled".to_string());
        }

        if self.length == 0 {
            return Err("Password length must be greater than 0".to_string());
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_set.len());
                character_set.chars().nth(idx).unwrap()
            })
            .collect();

        Ok(password)
    }
}

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn parse_bool_input(input: &str) -> bool {
    match input.to_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => true,
        _ => false,
    }
}

fn main() {
    println!("=== Secure Password Generator ===");
    
    let length_input = get_user_input("Enter password length (default: 16):");
    let length = if length_input.is_empty() {
        DEFAULT_LENGTH
    } else {
        length_input.parse().unwrap_or(DEFAULT_LENGTH)
    };

    let uppercase_input = get_user_input("Include uppercase letters? (Y/n):");
    let lowercase_input = get_user_input("Include lowercase letters? (Y/n):");
    let numbers_input = get_user_input("Include numbers? (Y/n):");
    let symbols_input = get_user_input("Include symbols? (Y/n):");

    let generator = PasswordGenerator::new()
        .with_length(length)
        .with_uppercase(parse_bool_input(&uppercase_input))
        .with_lowercase(parse_bool_input(&lowercase_input))
        .with_numbers(parse_bool_input(&numbers_input))
        .with_symbols(parse_bool_input(&symbols_input));

    match generator.generate() {
        Ok(password) => {
            println!("\nGenerated Password: {}", password);
            println!("Password Length: {}", password.len());
            println!("Character Sets Used:");
            println!("  Uppercase: {}", generator.use_uppercase);
            println!("  Lowercase: {}", generator.use_lowercase);
            println!("  Numbers: {}", generator.use_numbers);
            println!("  Symbols: {}", generator.use_symbols);
        }
        Err(e) => println!("Error: {}", e),
    }
}use rand::Rng;

pub struct PasswordGenerator {
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn with_digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn with_special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
        }

        if !self.use_lowercase && !self.use_uppercase && !self.use_digits && !self.use_special {
            return Err("At least one character set must be enabled");
        }

        let mut character_pool = Vec::new();
        let lowercase_chars = "abcdefghijklmnopqrstuvwxyz";
        let uppercase_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let digit_chars = "0123456789";
        let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";

        if self.use_lowercase {
            character_pool.extend(lowercase_chars.chars());
        }
        if self.use_uppercase {
            character_pool.extend(uppercase_chars.chars());
        }
        if self.use_digits {
            character_pool.extend(digit_chars.chars());
        }
        if self.use_special {
            character_pool.extend(special_chars.chars());
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx]
            })
            .collect();

        Ok(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_length() {
        let generator = PasswordGenerator::new(20);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_only_digits() {
        let generator = PasswordGenerator::new(8)
            .with_lowercase(false)
            .with_uppercase(false)
            .with_special(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_invalid_length() {
        let generator = PasswordGenerator::new(0);
        let result = generator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .with_lowercase(false)
            .with_uppercase(false)
            .with_digits(false)
            .with_special(false);
        let result = generator.generate();
        assert!(result.is_err());
    }
}