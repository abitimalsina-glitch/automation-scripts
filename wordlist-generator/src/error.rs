use std::fmt;

/// Application-level errors for keyspace-tool.
#[derive(Debug, PartialEq, Eq)]
pub enum AppError {
    MissingMode,
    EmptyCharset,
    MissingMinLength,
    MissingMaxLength,
    MissingLength,
    InvalidLength {
        name: &'static str,
        min: usize,
        max: usize,
        actual: usize,
    },
    InvalidLengthRange {
        min: usize,
        max: usize,
    },
    InvalidRate,
    MissingCount,
    MissingRecordLength,
    MissingOutputPath,
    InvalidCount,
    FileError {
        path: String,
        message: String,
    },
    InteractiveQuit,
    InputError {
        message: String,
    },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingMode => write!(
                f,
                "specify either --analyze, --generate, or --dataset.\nRun with --help for usage information."
            ),
            AppError::EmptyCharset => write!(
                f,
                "no characters selected. Use --lowercase, --uppercase, --digits, --symbols, or --custom."
            ),
            AppError::MissingMinLength => write!(f, "--min-length is required for --analyze."),
            AppError::MissingMaxLength => write!(f, "--max-length is required for --analyze."),
            AppError::MissingLength => write!(f, "--length is required for --generate."),
            AppError::InvalidLength {
                name,
                min,
                max,
                actual,
            } => {
                write!(f, "{name} must be between {min} and {max}, got {actual}")
            }
            AppError::InvalidLengthRange { min, max } => {
                write!(
                    f,
                    "--min-length ({min}) must not exceed --max-length ({max})."
                )
            }
            AppError::InvalidRate => write!(f, "--rate must be greater than 0."),
            AppError::MissingCount => write!(f, "--count is required for --dataset."),
            AppError::MissingRecordLength => {
                write!(f, "--record-length is required for --dataset.")
            }
            AppError::MissingOutputPath => write!(f, "--output is required for --dataset."),
            AppError::InvalidCount => write!(f, "--count must be at least 1."),
            AppError::FileError { path, message } => {
                write!(f, "file error for '{path}': {message}")
            }
            AppError::InteractiveQuit => write!(f, "session terminated."),
            AppError::InputError { message } => write!(f, "input error: {message}"),
        }
    }
}

impl std::error::Error for AppError {}
