use num_bigint::BigUint;
use num_traits::One;

pub struct SearchTime {
    pub best: BigUint,
    pub average: BigUint,
    pub worst: BigUint,
}

/// Estimate search times in seconds given a total keyspace and a guessing rate.
///
/// Formulas:
///   Best case    = 1 / rate  (first guess is correct; ≈ 0 for practical rates)
///   Average case = keyspace / (2 × rate)
///   Worst case   = keyspace / rate
///
/// Assumptions:
///   - Constant search rate with no computational overhead.
///   - Uniform random key distribution (all keys equally likely).
///   - No early-termination optimizations.
///   - The supplied rate is hypothetical, not measured hardware performance.
pub fn estimate_search_time(keyspace: &BigUint, rate: u64) -> SearchTime {
    let rate_big = BigUint::from(rate);

    SearchTime {
        best: BigUint::one() / &rate_big,
        average: keyspace / (BigUint::from(2u32) * &rate_big),
        worst: keyspace / &rate_big,
    }
}
