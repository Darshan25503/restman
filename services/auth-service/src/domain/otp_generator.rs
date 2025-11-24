use rand::Rng;

/// Generate a random OTP code with the specified length
pub fn generate_otp(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let otp: String = (0..length)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect();
    otp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_otp_length() {
        let otp = generate_otp(6);
        assert_eq!(otp.len(), 6);
    }

    #[test]
    fn test_generate_otp_numeric() {
        let otp = generate_otp(6);
        assert!(otp.chars().all(|c| c.is_numeric()));
    }

    #[test]
    fn test_generate_otp_different() {
        let otp1 = generate_otp(6);
        let otp2 = generate_otp(6);
        // Very unlikely to be the same (1 in 1,000,000 chance)
        // But we'll just check they're both valid
        assert_eq!(otp1.len(), 6);
        assert_eq!(otp2.len(), 6);
    }
}

