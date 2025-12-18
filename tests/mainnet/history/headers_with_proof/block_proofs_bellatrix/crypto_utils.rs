
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_secure_token(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_api_key() -> String {
    generate_secure_token(32)
}

pub fn generate_session_token() -> String {
    generate_secure_token(64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_token() {
        let token = generate_secure_token(16);
        assert_eq!(token.len(), 16);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_generate_session_token() {
        let token = generate_session_token();
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_tokens_are_unique() {
        let token1 = generate_secure_token(16);
        let token2 = generate_secure_token(16);
        assert_ne!(token1, token2);
    }
}