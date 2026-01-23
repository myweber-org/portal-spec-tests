use rand::{thread_rng, Rng};
use std::collections::HashSet;

pub struct KeyGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl KeyGenerator {
    pub fn new(length: usize) -> Self {
        KeyGenerator {
            length,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: false,
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
            return Err("Key length must be greater than zero");
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
            character_pool.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = thread_rng();
        let mut result = String::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        while result.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            
            if used_chars.insert(ch) || used_chars.len() >= character_pool.len() {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

pub fn generate_api_key() -> String {
    KeyGenerator::new(32)
        .uppercase(true)
        .lowercase(true)
        .digits(true)
        .special(false)
        .generate()
        .unwrap_or_else(|_| "default_key".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_length() {
        let key = KeyGenerator::new(16).generate().unwrap();
        assert_eq!(key.len(), 16);
    }

    #[test]
    fn test_character_sets() {
        let key = KeyGenerator::new(20)
            .uppercase(true)
            .lowercase(false)
            .digits(true)
            .special(false)
            .generate()
            .unwrap();
        
        assert!(key.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_invalid_config() {
        let result = KeyGenerator::new(10)
            .uppercase(false)
            .lowercase(false)
            .digits(false)
            .special(false)
            .generate();
        
        assert!(result.is_err());
    }
}