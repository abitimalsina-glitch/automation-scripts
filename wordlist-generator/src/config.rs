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
