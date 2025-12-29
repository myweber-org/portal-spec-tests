
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
            return Err("Password length must be greater than 0");
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
            character_pool.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars = Vec::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            
            if used_chars.insert(ch) || password_chars.len() >= character_pool.len().min(self.length) {
                password_chars.push(ch);
            }
        }

        Ok(password_chars.into_iter().collect())
    }
}

pub fn generate_secure_password(length: usize) -> String {
    PasswordGenerator::new(length)
        .generate()
        .unwrap_or_else(|_| "default_password".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_secure_password(12);
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_lowercase() || c.is_digit(10)));
    }

    #[test]
    fn test_invalid_configuration() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        
        assert!(generator.generate().is_err());
    }
}