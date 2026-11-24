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
