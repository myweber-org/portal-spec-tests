
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};

pub fn generate_random_token(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn verify_token(token: &str, hash: &str) -> bool {
    hash_token(token) == hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let token = generate_random_token(32);
        assert_eq!(token.len(), 32);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_token_hashing() {
        let token = "test_token_123";
        let hash = hash_token(token);
        assert_eq!(hash.len(), 64);
        assert!(verify_token(token, &hash));
        assert!(!verify_token("wrong_token", &hash));
    }
}