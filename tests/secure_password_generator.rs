use rand::Rng;

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
}
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
        PasswordGenerator {
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
            required_chars.insert(*b"abcdefghijklmnopqrstuvwxyz".choose(&mut rand::thread_rng()).unwrap() as char);
        }

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
            required_chars.insert(*b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".choose(&mut rand::thread_rng()).unwrap() as char);
        }

        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
            required_chars.insert(*b"0123456789".choose(&mut rand::thread_rng()).unwrap() as char);
        }

        if self.use_special {
            character_pool.extend(b"!@#$%^&*()-_=+[]{}|;:,.<>?/~".iter().copied());
            required_chars.insert(*b"!@#$%^&*".choose(&mut rand::thread_rng()).unwrap() as char);
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars: Vec<char> = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx] as char
            })
            .collect();

        for (i, required_char) in required_chars.into_iter().enumerate() {
            if i < password_chars.len() {
                password_chars[i] = required_char;
            }
        }

        for i in 0..password_chars.len() {
            let swap_idx = rng.gen_range(0..password_chars.len());
            password_chars.swap(i, swap_idx);
        }

        Ok(password_chars.into_iter().collect())
    }
}

pub fn generate_password(length: usize) -> Result<String, &'static str> {
    PasswordGenerator::new(length).generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
        let password = generate_password(12).unwrap();
        assert_eq!(password.len(), 12);
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_digit(10)));
        assert!(password.chars().any(|c| "!@#$%^&*()-_=+[]{}|;:,.<>?/~".contains(c)));
    }

    #[test]
    fn test_custom_generator() {
        let generator = PasswordGenerator::new(16)
            .uppercase(false)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_digit(10)));
        assert!(!password.chars().any(|c| c.is_uppercase()));
        assert!(!password.chars().any(|c| "!@#$%^&*()-_=+[]{}|;:,.<>?/~".contains(c)));
    }

    #[test]
    fn test_zero_length() {
        let result = generate_password(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        
        let result = generator.generate();
        assert!(result.is_err());
    }
}