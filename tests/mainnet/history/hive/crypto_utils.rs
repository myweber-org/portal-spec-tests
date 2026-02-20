
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub fn generate_secure_token() -> [u8; 32] {
    let mut token = [0u8; 32];
    thread_rng().fill(&mut token);
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(16);
        assert_eq!(password.len(), 16);
    }

    #[test]
    fn test_token_size() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 32);
    }
}
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_random_string(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_secure_token() -> String {
    generate_random_string(32)
}

pub fn generate_api_key() -> String {
    let prefix = "api_";
    let random_part = generate_random_string(24);
    format!("{}{}", prefix, random_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string_length() {
        let result = generate_random_string(16);
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn test_generate_secure_token_length() {
        let result = generate_secure_token();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_generate_api_key_format() {
        let result = generate_api_key();
        assert!(result.starts_with("api_"));
        assert_eq!(result.len(), 28);
    }

    #[test]
    fn test_random_strings_different() {
        let s1 = generate_random_string(10);
        let s2 = generate_random_string(10);
        assert_ne!(s1, s2);
    }
}