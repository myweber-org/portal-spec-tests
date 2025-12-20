use argon2::{self, Config, ThreadMode, Variant, Version};
use rand::{rngs::OsRng, RngCore};

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn hash_password(password: &str, salt: &[u8]) -> Result<String, argon2::Error> {
    let config = Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: 4096,
        time_cost: 3,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    
    let hash = argon2::hash_encoded(password.as_bytes(), salt, &config)?;
    Ok(hash)
}

pub fn verify_password(password: &str, encoded_hash: &str) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(encoded_hash, password.as_bytes())
}