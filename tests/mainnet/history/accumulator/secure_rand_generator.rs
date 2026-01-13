
use getrandom::getrandom;

pub fn generate_secure_random(bytes: &mut [u8]) -> Result<(), getrandom::Error> {
    getrandom(bytes)
}

pub fn generate_secure_u64() -> Result<u64, getrandom::Error> {
    let mut buffer = [0u8; 8];
    getrandom(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

pub fn generate_secure_u32() -> Result<u32, getrandom::Error> {
    let mut buffer = [0u8; 4];
    getrandom(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_random() {
        let mut buffer = [0u8; 32];
        let result = generate_secure_random(&mut buffer);
        assert!(result.is_ok());
        
        let mut sum = 0;
        for &byte in &buffer {
            sum += byte;
        }
        assert!(sum > 0);
    }

    #[test]
    fn test_generate_secure_u64() {
        let result = generate_secure_u64();
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_ne!(value, 0);
    }

    #[test]
    fn test_generate_secure_u32() {
        let result = generate_secure_u32();
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_ne!(value, 0);
    }
}