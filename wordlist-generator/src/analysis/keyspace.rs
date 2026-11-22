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

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::One;

    // ── Exact combination calculations ───────────────────────────────────

    #[test]
    fn combinations_26_pow_2() {
        assert_eq!(combinations(26, 2), BigUint::from(676u32));
    }

    #[test]
    fn combinations_10_pow_4() {
        assert_eq!(combinations(10, 4), BigUint::from(10_000u32));
    }

    #[test]
    fn combinations_length_1() {
        assert_eq!(combinations(62, 1), BigUint::from(62u32));
    }

    #[test]
    fn combinations_length_0_is_one() {
        // C^0 = 1 for any C.
        assert_eq!(combinations(26, 0), BigUint::one());
    }

    // ── Inclusive min-to-max calculations ─────────────────────────────────

    #[test]
    fn total_keyspace_26_range_1_to_2() {
        // 26^1 + 26^2 = 26 + 676 = 702
        let total = total_keyspace(26, 1, 2);
        assert_eq!(total, BigUint::from(702u32));
    }

    #[test]
    fn total_keyspace_single_length() {
        let total = total_keyspace(10, 4, 4);
        assert_eq!(total, BigUint::from(10_000u32));
    }

    #[test]
    fn total_keyspace_10_range_1_to_3() {
        // 10 + 100 + 1000 = 1110
        let total = total_keyspace(10, 1, 3);
        assert_eq!(total, BigUint::from(1_110u32));
    }

    // ── Arbitrary-precision arithmetic ───────────────────────────────────

    #[test]
    fn large_keyspace_exceeds_u128() {
        // 62^30 ≈ 5.9 × 10^53, vastly larger than u128::MAX ≈ 3.4 × 10^38.
        let result = combinations(62, 30);
        assert!(result > BigUint::from(u128::MAX));
    }

    #[test]
    fn very_large_exponent_no_panic() {
        // 95^256 — should compute without panic or overflow.
        let result = combinations(95, 256);
        assert!(result > BigUint::zero());
    }
}
