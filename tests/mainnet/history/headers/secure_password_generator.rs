
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

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
        let mut required_chars = Vec::new();

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
            required_chars.push(self.random_uppercase());
        }

        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
            required_chars.push(self.random_lowercase());
        }

        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
            required_chars.push(self.random_digit());
        }

        if self.use_special {
            character_pool.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
            required_chars.push(self.random_special());
        }

        let mut rng = thread_rng();
        let mut password_chars: Vec<char> = required_chars.into_iter().collect();

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            password_chars.push(character_pool[idx] as char);
        }

        rng.shuffle(&mut password_chars);

        Ok(password_chars.into_iter().collect())
    }

    fn random_uppercase(&self) -> char {
        let mut rng = thread_rng();
        rng.gen_range(b'A'..=b'Z') as char
    }

    fn random_lowercase(&self) -> char {
        let mut rng = thread_rng();
        rng.gen_range(b'a'..=b'z') as char
    }

    fn random_digit(&self) -> char {
        let mut rng = thread_rng();
        rng.gen_range(b'0'..=b'9') as char
    }

    fn random_special(&self) -> char {
        let special_chars = b"!@#$%^&*()_+-=[]{}|;:,.<>?";
        let mut rng = thread_rng();
        let idx = rng.gen_range(0..special_chars.len());
        special_chars[idx] as char
    }
}

pub fn generate_alphanumeric(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_password_generation() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_options() {
        let generator = PasswordGenerator::new(16)
            .uppercase(true)
            .lowercase(true)
            .digits(false)
            .special(false);

        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_alphanumeric_generator() {
        let password = generate_alphanumeric(10);
        assert_eq!(password.len(), 10);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_invalid_configuration() {
        let generator = PasswordGenerator::new(0);
        assert!(generator.generate().is_err());

        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .lowercase(false)
            .digits(false)
            .special(false);
        assert!(generator.generate().is_err());
    }
}