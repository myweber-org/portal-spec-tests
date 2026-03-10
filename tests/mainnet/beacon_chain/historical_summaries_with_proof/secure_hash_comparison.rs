use subtle::ConstantTimeEq;

pub fn verify_hash(expected: &[u8], actual: &[u8]) -> bool {
    expected.ct_eq(actual).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_matching_hashes() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(verify_hash(&hash1, &hash2));
    }

    #[test]
    fn test_different_hashes() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("d3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(!verify_hash(&hash1, &hash2));
    }

    #[test]
    fn test_empty_hashes() {
        assert!(verify_hash(&[], &[]));
    }

    #[test]
    fn test_different_lengths() {
        let short = hex!("e3b0c442");
        let long = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(!verify_hash(&short, &long));
    }
}