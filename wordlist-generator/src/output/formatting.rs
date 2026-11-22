use num_bigint::BigUint;
use num_traits::Zero;
use std::time::Duration;

/// Format a BigUint with comma-separated thousands for readability.
pub fn format_with_commas(n: &BigUint) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

/// Format a BigUint number of seconds into a human-readable duration string.
///
/// Uses the largest appropriate unit: seconds, minutes, hours, days, or years.
pub fn format_duration(seconds: &BigUint) -> String {
    if seconds.is_zero() {
        return "< 1 second".to_string();
    }

    let secs_per_minute = BigUint::from(60u32);
    let secs_per_hour = BigUint::from(3_600u32);
    let secs_per_day = BigUint::from(86_400u32);
    let secs_per_year = BigUint::from(31_557_600u32); // 365.25 days

    if *seconds < secs_per_minute {
        format!("{} seconds", format_with_commas(seconds))
    } else if *seconds < secs_per_hour {
        let mins = seconds / &secs_per_minute;
        format!("{} minutes", format_with_commas(&mins))
    } else if *seconds < secs_per_day {
        let hours = seconds / &secs_per_hour;
        format!("{} hours", format_with_commas(&hours))
    } else if *seconds < secs_per_year {
        let days = seconds / &secs_per_day;
        format!("{} days", format_with_commas(&days))
    } else {
        let years = seconds / &secs_per_year;
        format!("{} years", format_with_commas(&years))
    }
}

/// Format an elapsed execution time into a human-readable string with an appropriate unit:
/// nanoseconds (ns), microseconds (µs), milliseconds (ms), or seconds (s).
pub fn format_elapsed_time(duration: Duration) -> String {
    let nanos = duration.as_nanos();
    if nanos < 1_000 {
        format!("{nanos} ns")
    } else if nanos < 1_000_000 {
        let us = nanos as f64 / 1_000.0;
        format!("{:.2} µs", us)
    } else if nanos < 1_000_000_000 {
        let ms = nanos as f64 / 1_000_000.0;
        format!("{:.2} ms", ms)
    } else {
        let secs = duration.as_secs_f64();
        format!("{:.2} s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;

    #[test]
    fn format_commas_known_values() {
        assert_eq!(format_with_commas(&BigUint::from(0u32)), "0");
        assert_eq!(format_with_commas(&BigUint::from(999u32)), "999");
        assert_eq!(format_with_commas(&BigUint::from(1_000u32)), "1,000");
        assert_eq!(
            format_with_commas(&BigUint::from(1_000_000u32)),
            "1,000,000"
        );
    }

    #[test]
    fn format_duration_units() {
        assert_eq!(format_duration(&BigUint::zero()), "< 1 second");
        assert_eq!(format_duration(&BigUint::from(30u32)), "30 seconds");
        assert_eq!(format_duration(&BigUint::from(120u32)), "2 minutes");
        assert_eq!(format_duration(&BigUint::from(7_200u32)), "2 hours");
        assert_eq!(format_duration(&BigUint::from(172_800u32)), "2 days");
        assert_eq!(format_duration(&BigUint::from(63_115_200u32)), "2 years");
    }

    #[test]
    fn format_elapsed_time_nanoseconds() {
        assert_eq!(format_elapsed_time(Duration::from_nanos(0)), "0 ns");
        assert_eq!(format_elapsed_time(Duration::from_nanos(450)), "450 ns");
        assert_eq!(format_elapsed_time(Duration::from_nanos(999)), "999 ns");
    }

    #[test]
    fn format_elapsed_time_microseconds() {
        assert_eq!(format_elapsed_time(Duration::from_nanos(1_000)), "1.00 µs");
        assert_eq!(
            format_elapsed_time(Duration::from_nanos(12_450)),
            "12.45 µs"
        );
        assert_eq!(
            format_elapsed_time(Duration::from_nanos(999_900)),
            "999.90 µs"
        );
    }

    #[test]
    fn format_elapsed_time_milliseconds() {
        assert_eq!(
            format_elapsed_time(Duration::from_nanos(1_000_000)),
            "1.00 ms"
        );
        assert_eq!(
            format_elapsed_time(Duration::from_nanos(12_340_000)),
            "12.34 ms"
        );
        assert_eq!(format_elapsed_time(Duration::from_millis(500)), "500.00 ms");
    }

    #[test]
    fn format_elapsed_time_seconds() {
        assert_eq!(format_elapsed_time(Duration::from_secs(1)), "1.00 s");
        assert_eq!(format_elapsed_time(Duration::from_millis(2_500)), "2.50 s");
        assert_eq!(format_elapsed_time(Duration::from_secs(65)), "65.00 s");
    }
}
