use std::collections::BTreeSet;

// ─── Character Set Constants ─────────────────────────────────────────────────

pub const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
pub const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const DIGITS: &str = "0123456789";
pub const SYMBOLS: &str = "!@#$%^&*()-_=+[]{}|;:',.<>?/~\"\\`";

// ─── Length Constraints ──────────────────────────────────────────────────────

pub const MIN_PASSWORD_LENGTH: usize = 1;
pub const MAX_PASSWORD_LENGTH: usize = 256;

// ─── Character Set Construction ──────────────────────────────────────────────

/// Build a deduplicated, sorted character set from the selected categories.
pub fn build_charset(
    lowercase: bool,
    uppercase: bool,
    digits: bool,
    symbols: bool,
    custom: Option<&str>,
) -> Vec<char> {
    let mut set = BTreeSet::new();

    if lowercase {
        set.extend(LOWERCASE.chars());
    }
    if uppercase {
        set.extend(UPPERCASE.chars());
    }
    if digits {
        set.extend(DIGITS.chars());
    }
    if symbols {
        set.extend(SYMBOLS.chars());
    }
    if let Some(chars) = custom {
        for ch in chars.chars() {
            if ch.is_ascii() {
                set.insert(ch);
            }
        }
    }

    set.into_iter().collect()
}

// ─── Validation ──────────────────────────────────────────────────────────────

pub fn validate_length(value: usize, name: &str) -> Result<(), String> {
    if value < MIN_PASSWORD_LENGTH || value > MAX_PASSWORD_LENGTH {
        Err(format!(
            "{name} must be between {MIN_PASSWORD_LENGTH} and {MAX_PASSWORD_LENGTH}, got {value}"
        ))
    } else {
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Character-set construction ───────────────────────────────────────

    #[test]
    fn charset_lowercase_has_26_chars() {
        let cs = build_charset(true, false, false, false, None);
        assert_eq!(cs.len(), 26);
        assert!(cs.contains(&'a'));
        assert!(cs.contains(&'z'));
    }

    #[test]
    fn charset_uppercase_has_26_chars() {
        let cs = build_charset(false, true, false, false, None);
        assert_eq!(cs.len(), 26);
        assert!(cs.contains(&'A'));
        assert!(cs.contains(&'Z'));
    }

    #[test]
    fn charset_digits_has_10_chars() {
        let cs = build_charset(false, false, true, false, None);
        assert_eq!(cs.len(), 10);
        assert!(cs.contains(&'0'));
        assert!(cs.contains(&'9'));
    }

    #[test]
    fn charset_symbols_non_empty() {
        let cs = build_charset(false, false, false, true, None);
        assert!(!cs.is_empty());
        assert!(cs.contains(&'!'));
        assert!(cs.contains(&'@'));
    }

    #[test]
    fn charset_combined_lowercase_uppercase_digits() {
        let cs = build_charset(true, true, true, false, None);
        assert_eq!(cs.len(), 62);
    }

    #[test]
    fn charset_custom_only() {
        let cs = build_charset(false, false, false, false, Some("xyz123"));
        assert_eq!(cs.len(), 6);
    }

    // ── Duplicate-character removal ──────────────────────────────────────

    #[test]
    fn deduplication_in_custom_chars() {
        let cs = build_charset(false, false, false, false, Some("aaabbb"));
        assert_eq!(cs.len(), 2);
        assert!(cs.contains(&'a'));
        assert!(cs.contains(&'b'));
    }

    #[test]
    fn deduplication_overlap_with_category() {
        // Custom "abc" overlaps with lowercase — should still be 26.
        let cs = build_charset(true, false, false, false, Some("abc"));
        assert_eq!(cs.len(), 26);
    }

    // ── Empty character-set rejection ────────────────────────────────────

    #[test]
    fn empty_charset_when_nothing_selected() {
        let cs = build_charset(false, false, false, false, None);
        assert!(cs.is_empty());
    }

    // ── Length validation ────────────────────────────────────────────────

    #[test]
    fn validate_length_zero_rejected() {
        assert!(validate_length(0, "test").is_err());
    }

    #[test]
    fn validate_length_257_rejected() {
        assert!(validate_length(257, "test").is_err());
    }

    #[test]
    fn validate_length_boundaries_accepted() {
        assert!(validate_length(1, "test").is_ok());
        assert!(validate_length(128, "test").is_ok());
        assert!(validate_length(256, "test").is_ok());
    }
}
