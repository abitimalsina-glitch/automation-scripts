use crate::cli::Cli;
use crate::error::AppError;
use std::collections::BTreeSet;

// ─── Character Set Constants ─────────────────────────────────────────────────

pub const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
pub const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const DIGITS: &str = "0123456789";
pub const SYMBOLS: &str = "!@#$%^&*()-_=+[]{}|;:',.<>?/~\"\\`";

// ─── Length Constraints ──────────────────────────────────────────────────────

pub const MIN_PASSWORD_LENGTH: usize = 1;
pub const MAX_PASSWORD_LENGTH: usize = 256;

// ─── Application Mode & Config ───────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Analyze {
        min_length: usize,
        max_length: usize,
        rate: Option<u64>,
    },
    Generate {
        length: usize,
    },
    Dataset {
        count: u64,
        record_length: usize,
        output_path: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub charset: Vec<char>,
    pub mode: Mode,
}

impl Config {
    pub fn from_cli(cli: &Cli) -> Result<Self, AppError> {
        if !cli.analyze && !cli.generate && !cli.dataset {
            return Err(AppError::MissingMode);
        }

        let charset = build_charset(
            cli.lowercase,
            cli.uppercase,
            cli.digits,
            cli.symbols,
            cli.custom.as_deref(),
        );

        if charset.is_empty() {
            return Err(AppError::EmptyCharset);
        }

        let mode = if cli.analyze {
            let min_len = cli.min_length.ok_or(AppError::MissingMinLength)?;
            let max_len = cli.max_length.ok_or(AppError::MissingMaxLength)?;

            validate_length(min_len, "--min-length")?;
            validate_length(max_len, "--max-length")?;

            if min_len > max_len {
                return Err(AppError::InvalidLengthRange {
                    min: min_len,
                    max: max_len,
                });
            }

            if cli.rate == Some(0) {
                return Err(AppError::InvalidRate);
            }

            Mode::Analyze {
                min_length: min_len,
                max_length: max_len,
                rate: cli.rate,
            }
        } else if cli.generate {
            let length = cli.length.ok_or(AppError::MissingLength)?;
            validate_length(length, "--length")?;
            Mode::Generate { length }
        } else {
            // dataset mode
            let count = cli.count.ok_or(AppError::MissingCount)?;
            if count == 0 {
                return Err(AppError::InvalidCount);
            }

            let record_length = cli.record_length.ok_or(AppError::MissingRecordLength)?;
            validate_length(record_length, "--record-length")?;

            let output_path = cli.output.clone().ok_or(AppError::MissingOutputPath)?;

            Mode::Dataset {
                count,
                record_length,
                output_path,
            }
        };

        Ok(Config { charset, mode })
    }
}

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

pub fn validate_length(value: usize, name: &'static str) -> Result<(), AppError> {
    if !(MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH).contains(&value) {
        Err(AppError::InvalidLength {
            name,
            min: MIN_PASSWORD_LENGTH,
            max: MAX_PASSWORD_LENGTH,
            actual: value,
        })
    } else {
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to build a Cli struct with all defaults.
    fn default_cli() -> Cli {
        Cli {
            analyze: false,
            generate: false,
            dataset: false,
            min_length: None,
            max_length: None,
            length: None,
            lowercase: false,
            uppercase: false,
            digits: false,
            symbols: false,
            custom: None,
            rate: None,
            count: None,
            record_length: None,
            output: None,
        }
    }

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

    // ── Config::from_cli validation ──────────────────────────────────────

    #[test]
    fn config_missing_mode() {
        let cli = Cli {
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::MissingMode));
    }

    #[test]
    fn config_empty_charset() {
        let cli = Cli {
            analyze: true,
            min_length: Some(8),
            max_length: Some(16),
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::EmptyCharset));
    }

    #[test]
    fn config_analyze_valid() {
        let cli = Cli {
            analyze: true,
            min_length: Some(8),
            max_length: Some(16),
            lowercase: true,
            rate: Some(1000),
            ..default_cli()
        };
        let cfg = Config::from_cli(&cli).unwrap();
        assert_eq!(cfg.charset.len(), 26);
        assert_eq!(
            cfg.mode,
            Mode::Analyze {
                min_length: 8,
                max_length: 16,
                rate: Some(1000)
            }
        );
    }

    #[test]
    fn config_generate_valid() {
        let cli = Cli {
            generate: true,
            length: Some(20),
            lowercase: true,
            uppercase: true,
            ..default_cli()
        };
        let cfg = Config::from_cli(&cli).unwrap();
        assert_eq!(cfg.charset.len(), 52);
        assert_eq!(cfg.mode, Mode::Generate { length: 20 });
    }

    #[test]
    fn config_analyze_missing_lengths() {
        let cli = Cli {
            analyze: true,
            max_length: Some(16),
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::MissingMinLength));

        let cli = Cli {
            analyze: true,
            min_length: Some(8),
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::MissingMaxLength));
    }

    #[test]
    fn config_analyze_inverted_range() {
        let cli = Cli {
            analyze: true,
            min_length: Some(16),
            max_length: Some(8),
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(
            Config::from_cli(&cli),
            Err(AppError::InvalidLengthRange { min: 16, max: 8 })
        );
    }

    #[test]
    fn config_analyze_zero_rate() {
        let cli = Cli {
            analyze: true,
            min_length: Some(8),
            max_length: Some(16),
            lowercase: true,
            rate: Some(0),
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::InvalidRate));
    }

    #[test]
    fn config_generate_missing_length() {
        let cli = Cli {
            generate: true,
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::MissingLength));
    }

    // ── Dataset mode Config validation ───────────────────────────────────

    #[test]
    fn config_dataset_valid() {
        let cli = Cli {
            dataset: true,
            count: Some(1000),
            record_length: Some(16),
            output: Some("/tmp/test.txt".to_string()),
            lowercase: true,
            uppercase: true,
            digits: true,
            ..default_cli()
        };
        let cfg = Config::from_cli(&cli).unwrap();
        assert_eq!(cfg.charset.len(), 62);
        assert_eq!(
            cfg.mode,
            Mode::Dataset {
                count: 1000,
                record_length: 16,
                output_path: "/tmp/test.txt".to_string(),
            }
        );
    }

    #[test]
    fn config_dataset_missing_count() {
        let cli = Cli {
            dataset: true,
            record_length: Some(16),
            output: Some("/tmp/test.txt".to_string()),
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::MissingCount));
    }

    #[test]
    fn config_dataset_zero_count() {
        let cli = Cli {
            dataset: true,
            count: Some(0),
            record_length: Some(16),
            output: Some("/tmp/test.txt".to_string()),
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::InvalidCount));
    }

    #[test]
    fn config_dataset_missing_record_length() {
        let cli = Cli {
            dataset: true,
            count: Some(1000),
            output: Some("/tmp/test.txt".to_string()),
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::MissingRecordLength));
    }

    #[test]
    fn config_dataset_invalid_record_length() {
        let cli = Cli {
            dataset: true,
            count: Some(1000),
            record_length: Some(0),
            output: Some("/tmp/test.txt".to_string()),
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(
            Config::from_cli(&cli),
            Err(AppError::InvalidLength {
                name: "--record-length",
                min: 1,
                max: 256,
                actual: 0,
            })
        );
    }

    #[test]
    fn config_dataset_missing_output() {
        let cli = Cli {
            dataset: true,
            count: Some(1000),
            record_length: Some(16),
            lowercase: true,
            ..default_cli()
        };
        assert_eq!(Config::from_cli(&cli), Err(AppError::MissingOutputPath));
    }
}
