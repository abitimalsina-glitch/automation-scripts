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
