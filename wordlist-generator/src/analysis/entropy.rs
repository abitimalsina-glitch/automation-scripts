/// Compute entropy in bits: L × log₂(C).
///
/// For charset_size ≤ 1, entropy is 0 since there is at most one possible
/// value per position.
pub fn entropy_bits(charset_size: usize, length: usize) -> f64 {
    if charset_size <= 1 {
        return 0.0;
    }
    length as f64 * (charset_size as f64).log2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_26_length_2() {
        let e = entropy_bits(26, 2);
        let expected = 2.0 * (26.0_f64).log2(); // ≈ 9.40
        assert!((e - expected).abs() < 1e-10);
    }

    #[test]
    fn entropy_62_length_8() {
        let e = entropy_bits(62, 8);
        let expected = 8.0 * (62.0_f64).log2(); // ≈ 47.63
        assert!((e - expected).abs() < 1e-10);
    }

    #[test]
    fn entropy_charset_1_is_zero() {
        // log2(1) = 0, so entropy is always 0.
        assert_eq!(entropy_bits(1, 10), 0.0);
    }
}
