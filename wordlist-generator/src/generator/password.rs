use rand::Rng;
use std::time::{Duration, Instant};

/// Generate a cryptographically secure random password.
///
/// Uses the system CSPRNG (ChaCha12 seeded from OS entropy) via `rand::rng()`.
/// The `random_range` method employs rejection sampling to avoid modulo bias.
pub fn generate_password(charset: &[char], length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| charset[rng.random_range(0..charset.len())])
        .collect()
}

/// Generate a cryptographically secure random password while measuring elapsed execution time.
/// Starts timing immediately before password generation begins and stops immediately after it completes.
pub fn generate_password_timed(charset: &[char], length: usize) -> (String, Duration) {
    let start = Instant::now();
    let password = generate_password(charset, length);
    let elapsed = start.elapsed();
    (password, elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::build_charset;

    #[test]
    fn generated_password_has_correct_length() {
        let cs = build_charset(true, true, true, false, None);
        let pw = generate_password(&cs, 20);
        assert_eq!(pw.chars().count(), 20);
    }

    #[test]
    fn generated_password_uses_only_charset_chars() {
        let cs = build_charset(false, false, true, false, None); // digits only
        let pw = generate_password(&cs, 100);
        for ch in pw.chars() {
            assert!(cs.contains(&ch), "unexpected char '{ch}' not in charset");
        }
    }

    #[test]
    fn generated_password_is_not_empty() {
        let cs = build_charset(true, false, false, false, None);
        assert!(!generate_password(&cs, 10).is_empty());
    }

    #[test]
    fn generated_password_exact_lengths() {
        let cs = build_charset(true, true, true, true, None);
        for len in [1, 8, 16, 64, 128, 256] {
            assert_eq!(generate_password(&cs, len).chars().count(), len);
        }
    }

    #[test]
    fn test_timing_bounds_and_non_negative() {
        let cs = build_charset(true, true, true, false, None);
        let before = Instant::now();
        let (pw, elapsed) = generate_password_timed(&cs, 50);
        let after = Instant::now();

        assert_eq!(pw.chars().count(), 50);
        assert!(elapsed >= Duration::ZERO);
        assert!(elapsed <= after.duration_since(before));
    }
}
