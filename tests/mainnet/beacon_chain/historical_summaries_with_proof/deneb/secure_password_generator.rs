
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
use rand::Rng;

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

        let mut character_set = Vec::new();
        
        if self.use_lowercase {
            character_set.extend(b'a'..=b'z');
        }
        if self.use_uppercase {
            character_set.extend(b'A'..=b'Z');
        }
        if self.use_digits {
            character_set.extend(b'0'..=b'9');
        }
        if self.use_special {
            character_set.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        if character_set.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_set.len());
                character_set[idx] as char
            })
            .collect();

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
        let generator = PasswordGenerator::new(16);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 16);
    }

    #[test]
    fn test_only_digits() {
        let generator = PasswordGenerator::new(8)
            .lowercase(false)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(8)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        let result = generator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_length() {
        let generator = PasswordGenerator::new(0);
        let result = generator.generate();
        assert!(result.is_err());
    }
}use rand::Rng;
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

    pub fn generate(&self) -> Result<String, String> {
        if self.length == 0 {
            return Err("Password length must be greater than 0".to_string());
        }

        let mut character_pool = Vec::new();
        let mut required_chars = Vec::new();

        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
            required_chars.push(self.random_char_from_range(b'a'..=b'z'));
        }

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
            required_chars.push(self.random_char_from_range(b'A'..=b'Z'));
        }

        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
            required_chars.push(self.random_char_from_range(b'0'..=b'9'));
        }

        if self.use_special {
            let special_chars = b"!@#$%^&*()-_=+[]{}|;:,.<>?";
            character_pool.extend_from_slice(special_chars);
            required_chars.push(self.random_char_from_slice(special_chars));
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled".to_string());
        }

        let mut rng = rand::thread_rng();
        let mut password_chars: Vec<char> = required_chars
            .into_iter()
            .map(|b| b as char)
            .collect();

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            password_chars.push(character_pool[idx] as char);
        }

        Self::shuffle(&mut password_chars);
        Ok(password_chars.into_iter().collect())
    }

    fn random_char_from_range<R: rand::distributions::uniform::SampleRange<u8>>(&self, range: R) -> u8 {
        let mut rng = rand::thread_rng();
        rng.gen_range(range)
    }

    fn random_char_from_slice(&self, slice: &[u8]) -> u8 {
        let mut rng = rand::thread_rng();
        slice[rng.gen_range(0..slice.len())]
    }

    fn shuffle<T>(items: &mut [T]) {
        let mut rng = rand::thread_rng();
        for i in (1..items.len()).rev() {
            let j = rng.gen_range(0..=i);
            items.swap(i, j);
        }
    }

    pub fn validate_strength(password: &str) -> (bool, HashSet<&'static str>) {
        let mut issues = HashSet::new();
        
        if password.len() < 8 {
            issues.insert("Password must be at least 8 characters long");
        }
        
        let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());
        
        if !has_lowercase {
            issues.insert("Password must contain at least one lowercase letter");
        }
        if !has_uppercase {
            issues.insert("Password must contain at least one uppercase letter");
        }
        if !has_digit {
            issues.insert("Password must contain at least one digit");
        }
        if !has_special {
            issues.insert("Password must contain at least one special character");
        }
        
        (issues.is_empty(), issues)
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
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_strength_validation() {
        let strong_password = "SecureP@ssw0rd!";
        let (is_strong, issues) = PasswordGenerator::validate_strength(strong_password);
        assert!(is_strong);
        assert!(issues.is_empty());
        
        let weak_password = "weak";
        let (is_strong, issues) = PasswordGenerator::validate_strength(weak_password);
        assert!(!is_strong);
        assert!(!issues.is_empty());
    }
}