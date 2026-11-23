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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            AppError::MissingMode.to_string(),
            "specify either --analyze, --generate, or --dataset.\nRun with --help for usage information."
        );
        assert_eq!(
            AppError::EmptyCharset.to_string(),
            "no characters selected. Use --lowercase, --uppercase, --digits, --symbols, or --custom."
        );
        assert_eq!(
            AppError::MissingMinLength.to_string(),
            "--min-length is required for --analyze."
        );
        assert_eq!(
            AppError::MissingMaxLength.to_string(),
            "--max-length is required for --analyze."
        );
        assert_eq!(
            AppError::MissingLength.to_string(),
            "--length is required for --generate."
        );
        assert_eq!(
            AppError::InvalidLength {
                name: "--length",
                min: 1,
                max: 256,
                actual: 0
            }
            .to_string(),
            "--length must be between 1 and 256, got 0"
        );
        assert_eq!(
            AppError::InvalidLengthRange { min: 10, max: 5 }.to_string(),
            "--min-length (10) must not exceed --max-length (5)."
        );
        assert_eq!(
            AppError::InvalidRate.to_string(),
            "--rate must be greater than 0."
        );
    }

    #[test]
    fn test_dataset_error_display() {
        assert_eq!(
            AppError::MissingCount.to_string(),
            "--count is required for --dataset."
        );
        assert_eq!(
            AppError::MissingRecordLength.to_string(),
            "--record-length is required for --dataset."
        );
        assert_eq!(
            AppError::MissingOutputPath.to_string(),
            "--output is required for --dataset."
        );
        assert_eq!(
            AppError::InvalidCount.to_string(),
            "--count must be at least 1."
        );
        assert_eq!(
            AppError::FileError {
                path: "/tmp/test.txt".to_string(),
                message: "permission denied".to_string(),
            }
            .to_string(),
            "file error for '/tmp/test.txt': permission denied"
        );
    }

    #[test]
    fn test_interactive_error_display() {
        assert_eq!(AppError::InteractiveQuit.to_string(), "session terminated.");
        assert_eq!(
            AppError::InputError {
                message: "broken pipe".to_string()
            }
            .to_string(),
            "input error: broken pipe"
        );
    }
}
