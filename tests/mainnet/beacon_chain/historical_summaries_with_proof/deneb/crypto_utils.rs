
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string_length() {
        let result = generate_random_string(10);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_generate_secure_token_length() {
        let result = generate_secure_token();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_random_strings_different() {
        let s1 = generate_random_string(10);
        let s2 = generate_random_string(10);
        assert_ne!(s1, s2);
    }
}