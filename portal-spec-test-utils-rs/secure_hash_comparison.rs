use subtle::ConstantTimeEq;

pub fn verify_hash(expected: &[u8], candidate: &[u8]) -> bool {
    expected.ct_eq(candidate).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_hash_verification() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash3 = hex!("d3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

        assert!(verify_hash(&hash1, &hash2));
        assert!(!verify_hash(&hash1, &hash3));
    }

    #[test]
    fn test_different_lengths() {
        let short = b"short";
        let long = b"longer_hash";

        assert!(!verify_hash(short, long));
    }
}