use num_bigint::BigUint;
use num_traits::Zero;

/// Compute C^L (combinations at a given length) using arbitrary-precision
/// arithmetic.
pub fn combinations(charset_size: usize, length: usize) -> BigUint {
    num_traits::pow::pow(BigUint::from(charset_size), length)
}

/// Compute the total keyspace: sum of C^L for L in [min_len, max_len].
pub fn total_keyspace(charset_size: usize, min_len: usize, max_len: usize) -> BigUint {
    let mut total = BigUint::zero();
    for len in min_len..=max_len {
        total += combinations(charset_size, len);
    }
    total
}
