use rand::Rng;
use std::collections::HashSet;

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

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
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
            return Err("Password length must be greater than zero");
        }

        let mut character_pool = Vec::new();
        let mut required_chars = HashSet::new();

        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
            required_chars.insert(self.random_char_from_range(b'a'..=b'z'));
        }

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
            required_chars.insert(self.random_char_from_range(b'A'..=b'Z'));
        }

        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
            required_chars.insert(self.random_char_from_range(b'0'..=b'9'));
        }

        if self.use_special {
            character_pool.extend(b'!'..=b'/');
            character_pool.extend(b':'..=b'@');
            character_pool.extend(b'['..=b'`');
            character_pool.extend(b'{'..=b'~');
            required_chars.insert(self.random_special_char());
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars: Vec<char> = required_chars.into_iter().collect();

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            password_chars.push(character_pool[idx] as char);
        }

        password_chars.shuffle(&mut rng);
        Ok(password_chars.into_iter().collect())
    }

    fn random_char_from_range<R: rand::distributions::uniform::SampleRange<u8>>(&self, range: R) -> char {
        let mut rng = rand::thread_rng();
        rng.gen_range(range) as char
    }

    fn random_special_char(&self) -> char {
        let special_ranges = [
            b'!'..=b'/',
            b':'..=b'@',
            b'['..=b'`',
            b'{'..=b'~',
        ];
        let mut rng = rand::thread_rng();
        let range_idx = rng.gen_range(0..special_ranges.len());
        rng.gen_range(special_ranges[range_idx].clone()) as char
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_lowercase() || c.is_digit()));
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        assert!(generator.generate().is_err());
    }
}use rand::Rng;

pub fn generate_password(length: usize, use_uppercase: bool, use_numbers: bool, use_symbols: bool) -> String {
    let mut charset = String::from("abcdefghijklmnopqrstuvwxyz");
    
    if use_uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if use_numbers {
        charset.push_str("0123456789");
    }
    if use_symbols {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }
    
    let charset_bytes = charset.as_bytes();
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(12, true, true, true);
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_lowercase_only() {
        let password = generate_password(8, false, false, false);
        assert!(password.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_contains_uppercase() {
        let password = generate_password(10, true, false, false);
        assert!(password.chars().any(|c| c.is_uppercase()));
    }
}use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordConfig {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_symbols: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        PasswordConfig {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }
}

fn generate_password(config: &PasswordConfig) -> String {
    let mut character_pool = String::new();
    
    if config.use_uppercase {
        character_pool.push_str(UPPERCASE);
    }
    if config.use_lowercase {
        character_pool.push_str(LOWERCASE);
    }
    if config.use_numbers {
        character_pool.push_str(NUMBERS);
    }
    if config.use_symbols {
        character_pool.push_str(SYMBOLS);
    }
    
    if character_pool.is_empty() {
        return String::from("Error: No character sets selected");
    }
    
    let mut rng = rand::thread_rng();
    let password: String = (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..character_pool.len());
            character_pool.chars().nth(idx).unwrap()
        })
        .collect();
    
    password
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
    println!("Secure Password Generator");
    println!("=========================");
    
    let mut config = PasswordConfig::default();
    
    let length_input = get_user_input(&format!("Password length (default: {}): ", DEFAULT_LENGTH));
    if !length_input.is_empty() {
        if let Ok(length) = length_input.parse::<usize>() {
            if length >= 4 && length <= 128 {
                config.length = length;
            } else {
                println!("Length must be between 4 and 128. Using default.");
            }
        }
    }
    
    let uppercase_input = get_user_input("Include uppercase letters? (Y/n): ");
    if !uppercase_input.is_empty() {
        config.use_uppercase = parse_bool_input(&uppercase_input);
    }
    
    let lowercase_input = get_user_input("Include lowercase letters? (Y/n): ");
    if !lowercase_input.is_empty() {
        config.use_lowercase = parse_bool_input(&lowercase_input);
    }
    
    let numbers_input = get_user_input("Include numbers? (Y/n): ");
    if !numbers_input.is_empty() {
        config.use_numbers = parse_bool_input(&numbers_input);
    }
    
    let symbols_input = get_user_input("Include symbols? (Y/n): ");
    if !symbols_input.is_empty() {
        config.use_symbols = parse_bool_input(&symbols_input);
    }
    
    let password = generate_password(&config);
    println!("\nGenerated Password: {}", password);
    println!("Password Length: {}", password.len());
    
    let mut char_types = Vec::new();
    if config.use_uppercase { char_types.push("Uppercase"); }
    if config.use_lowercase { char_types.push("Lowercase"); }
    if config.use_numbers { char_types.push("Numbers"); }
    if config.use_symbols { char_types.push("Symbols"); }
    
    println!("Character sets used: {}", char_types.join(", "));
}