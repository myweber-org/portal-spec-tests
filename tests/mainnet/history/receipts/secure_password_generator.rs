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
        PasswordGenerator {
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
            return Err("Password length must be greater than zero");
        }

        let mut character_pool = Vec::new();
        
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
        }
        
        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
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

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password = String::with_capacity(self.length);

        for _ in 0..self.length {
            let idx = rng.gen_range(0..character_pool.len());
            password.push(character_pool[idx] as char);
        }

        Ok(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
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
    fn test_zero_length() {
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