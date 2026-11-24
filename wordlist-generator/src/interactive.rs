//! Interactive terminal configuration flow for keyspace-tool.
//!
//! When the application is run without command-line arguments, this module
//! guides the user through configuration interactively and returns a validated
//! `Config` that is executed by the same application logic used by the CLI path.

use crate::config::{
    Config, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH, Mode, build_charset, validate_length,
};
use crate::error::AppError;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Run the interactive configuration flow using stdin/stdout.
pub fn run_interactive() -> Result<Config, AppError> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    run_session(&mut stdin.lock(), &mut stdout.lock())
}

// ─── Core Session ────────────────────────────────────────────────────────────

/// Interactive session parameterized over I/O for testability.
pub(crate) fn run_session(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<Config, AppError> {
    display_banner(writer);

    loop {
        let mode = prompt_main_menu(reader, writer)?;
        let result = match mode {
            1 => collect_generate_config(reader, writer),
            2 => collect_analyze_config(reader, writer),
            _ => unreachable!(),
        };

        match result {
            Ok(Some(config)) => return Ok(config),
            Ok(None) => continue, // user chose "back"
            Err(e) => return Err(e),
        }
    }
}

// ─── Display ─────────────────────────────────────────────────────────────────

fn display_banner(writer: &mut impl Write) {
    writeln_to(writer, "");
    writeln_to(writer, "╔════════════════════════════════════════════════╗");
    writeln_to(writer, "║                 KEYSPACE TOOL                  ║");
    writeln_to(writer, "╚════════════════════════════════════════════════╝");
    writeln_to(writer, "");
}

// ─── Menu Flows ──────────────────────────────────────────────────────────────

fn prompt_main_menu(reader: &mut impl BufRead, writer: &mut impl Write) -> Result<usize, AppError> {
    writeln_to(writer, "What would you like to do?");
    writeln_to(writer, "");
    writeln_to(writer, "  1. Generate");
    writeln_to(writer, "  2. Calculate keyspace");
    writeln_to(writer, "");
    prompt_selection(reader, writer, "Select an option: ", 1, 2)
}

fn collect_generate_config(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<Option<Config>, AppError> {
    writeln_to(writer, "");
    writeln_to(writer, "Select generation mode:");
    writeln_to(writer, "");
    writeln_to(writer, "  1. Single password");
    writeln_to(writer, "  2. Synthetic dataset");
    writeln_to(writer, "  0. Back");
    writeln_to(writer, "");

    match prompt_selection_with_back(reader, writer, "Select an option: ", 2)? {
        None => Ok(None),
        Some(1) => collect_single_password(reader, writer),
        Some(2) => collect_dataset(reader, writer),
        _ => unreachable!(),
    }
}

fn collect_single_password(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<Option<Config>, AppError> {
    writeln_to(writer, "");
    writeln_to(writer, "── Single Password Generation ──");
    writeln_to(writer, "");

    let length = prompt_validated_length(reader, writer, "Password length")?;
    let charset = match prompt_charset(reader, writer)? {
        None => return Ok(None),
        Some(cs) => cs,
    };

    Ok(Some(Config {
        charset,
        mode: Mode::Generate { length },
    }))
}

fn collect_dataset(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<Option<Config>, AppError> {
    writeln_to(writer, "");
    writeln_to(writer, "── Synthetic Dataset Generation ──");
    writeln_to(writer, "");

    let count = prompt_positive_u64(reader, writer, "Number of records")?;
    let record_length = prompt_validated_length(reader, writer, "Record length")?;

    writeln_to(writer, "");
    let output_dir = prompt_output_directory(reader, writer)?;
    let output_filename = prompt_output_filename(reader, writer)?;
    let output_path = output_dir
        .join(output_filename)
        .to_string_lossy()
        .into_owned();

    let charset = match prompt_charset(reader, writer)? {
        None => return Ok(None),
        Some(cs) => cs,
    };

    Ok(Some(Config {
        charset,
        mode: Mode::Dataset {
            count,
            record_length,
            output_path,
        },
    }))
}

fn collect_analyze_config(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<Option<Config>, AppError> {
    writeln_to(writer, "");
    writeln_to(writer, "── Keyspace Analysis ──");
    writeln_to(writer, "");

    let min_length = prompt_validated_length(reader, writer, "Minimum length")?;
    let max_length = prompt_validated_length(reader, writer, "Maximum length")?;

    if min_length > max_length {
        writeln_to(
            writer,
            &format!(
                "\n  Error: minimum length ({min_length}) must not exceed maximum length ({max_length}).\n"
            ),
        );
        return Ok(None);
    }

    let charset = match prompt_charset(reader, writer)? {
        None => return Ok(None),
        Some(cs) => cs,
    };

    writeln_to(writer, "");
    let rate = prompt_optional_u64(reader, writer, "Guess rate (guesses/sec, Enter to skip): ")?;

    if rate == Some(0) {
        writeln_to(writer, "\n  Error: guess rate must be greater than 0.\n");
        return Ok(None);
    }

    Ok(Some(Config {
        charset,
        mode: Mode::Analyze {
            min_length,
            max_length,
            rate,
        },
    }))
}

// ─── Character Set Selection ─────────────────────────────────────────────────

fn prompt_charset(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<Option<Vec<char>>, AppError> {
    loop {
        writeln_to(writer, "");
        writeln_to(
            writer,
            "Select character categories (comma-separated, e.g., 1,2,3):",
        );
        writeln_to(writer, "");
        writeln_to(writer, "  1. Lowercase (a-z)");
        writeln_to(writer, "  2. Uppercase (A-Z)");
        writeln_to(writer, "  3. Digits (0-9)");
        writeln_to(writer, "  4. Symbols (!@#$%...)");
        writeln_to(writer, "  0. Back");
        writeln_to(writer, "");

        write_to(writer, "Selection: ");
        let line = read_line(reader)?;

        if line.trim() == "0" {
            return Ok(None);
        }

        let selections = parse_category_selections(line.trim());
        if selections.is_empty() {
            writeln_to(
                writer,
                "  Invalid selection. Enter comma-separated numbers (1-4).",
            );
            continue;
        }

        let charset = build_charset(
            selections.contains(&1),
            selections.contains(&2),
            selections.contains(&3),
            selections.contains(&4),
            None,
        );

        if charset.is_empty() {
            writeln_to(writer, "  No valid categories selected. Please try again.");
            continue;
        }

        return Ok(Some(charset));
    }
}

/// Parse comma-separated category numbers, returning valid selections (1–4).
pub(crate) fn parse_category_selections(input: &str) -> Vec<usize> {
    input
        .split(',')
        .filter_map(|s| s.trim().parse::<usize>().ok())
        .filter(|n| (1..=4).contains(n))
        .collect()
}

// ─── Input Primitives ────────────────────────────────────────────────────────

/// Read one line. Returns `Err(InteractiveQuit)` on EOF.
fn read_line(reader: &mut impl BufRead) -> Result<String, AppError> {
    let mut buf = String::new();
    let bytes = reader
        .read_line(&mut buf)
        .map_err(|e| AppError::InputError {
            message: e.to_string(),
        })?;
    if bytes == 0 {
        return Err(AppError::InteractiveQuit);
    }
    Ok(buf)
}

/// Prompt for a selection in `[min, max]`, re-prompting on invalid input.
fn prompt_selection(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    prompt: &str,
    min: usize,
    max: usize,
) -> Result<usize, AppError> {
    loop {
        write_to(writer, prompt);
        let line = read_line(reader)?;
        match line.trim().parse::<usize>() {
            Ok(n) if (min..=max).contains(&n) => return Ok(n),
            _ => writeln_to(
                writer,
                &format!("  Invalid selection. Enter a number between {min} and {max}."),
            ),
        }
    }
}

/// Prompt for a selection in `[1, max]`, with `0` meaning "back".
fn prompt_selection_with_back(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    prompt: &str,
    max: usize,
) -> Result<Option<usize>, AppError> {
    loop {
        write_to(writer, prompt);
        let line = read_line(reader)?;
        match line.trim().parse::<usize>() {
            Ok(0) => return Ok(None),
            Ok(n) if (1..=max).contains(&n) => return Ok(Some(n)),
            _ => writeln_to(writer, &format!("  Invalid selection. Enter 0-{max}.")),
        }
    }
}

/// Prompt for a password/record length, using existing validation constraints.
fn prompt_validated_length(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    label: &str,
) -> Result<usize, AppError> {
    loop {
        write_to(
            writer,
            &format!("{label} ({MIN_PASSWORD_LENGTH}-{MAX_PASSWORD_LENGTH}): "),
        );
        let line = read_line(reader)?;
        match line.trim().parse::<usize>() {
            Ok(n) if validate_length(n, "value").is_ok() => return Ok(n),
            Ok(_) => writeln_to(
                writer,
                &format!(
                    "  Value must be between {MIN_PASSWORD_LENGTH} and {MAX_PASSWORD_LENGTH}."
                ),
            ),
            Err(_) => writeln_to(writer, "  Please enter a valid number."),
        }
    }
}

/// Prompt for a positive u64 (≥ 1).
fn prompt_positive_u64(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    label: &str,
) -> Result<u64, AppError> {
    loop {
        write_to(writer, &format!("{label}: "));
        let line = read_line(reader)?;
        match line.trim().parse::<u64>() {
            Ok(n) if n >= 1 => return Ok(n),
            Ok(_) => writeln_to(writer, "  Value must be at least 1."),
            Err(_) => writeln_to(writer, "  Please enter a valid number."),
        }
    }
}

/// Prompt for an optional u64. Empty input returns `None`.
fn prompt_optional_u64(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    prompt: &str,
) -> Result<Option<u64>, AppError> {
    loop {
        write_to(writer, prompt);
        let line = read_line(reader)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        match trimmed.parse::<u64>() {
            Ok(n) => return Ok(Some(n)),
            Err(_) => writeln_to(
                writer,
                "  Please enter a valid number or press Enter to skip.",
            ),
        }
    }
}

/// Validate an output directory: must exist, be a directory, and be writable.
pub(crate) fn validate_output_directory(dir_str: &str) -> Result<PathBuf, String> {
    let trimmed = dir_str.trim();
    if trimmed.is_empty() {
        return Err("Directory path cannot be empty.".to_string());
    }
    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(format!("Directory '{}' does not exist.", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Path '{}' is not a directory.", path.display()));
    }
    check_dir_writable(&path)
        .map_err(|e| format!("Directory '{}' is not writable: {e}", path.display()))?;

    Ok(path)
}

/// Check that a directory can be used for writing by probing with a temporary file.
pub(crate) fn check_dir_writable(path: &Path) -> Result<(), String> {
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if metadata.permissions().readonly() {
        return Err("permission denied (directory is read-only)".to_string());
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let probe_path = path.join(format!(
        ".keyspace_probe_{}_{}.tmp",
        std::process::id(),
        nanos
    ));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe_path)
    {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe_path);
            Ok(())
        }
        Err(e) => Err(format!("permission denied ({e})")),
    }
}

/// Validate an output filename: non-empty, single normal component, no path separators or traversal.
pub(crate) fn validate_filename(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Filename cannot be empty.".to_string());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('\0') {
        return Err("Filename must not contain path separators or null bytes.".to_string());
    }
    if trimmed == "." || trimmed == ".." {
        return Err("Filename cannot be '.' or '..'.".to_string());
    }
    let path = Path::new(trimmed);
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(std::path::Component::Normal(_)), None) => Ok(trimmed.to_string()),
        _ => Err("Invalid filename.".to_string()),
    }
}

/// Prompt for a valid, writable output directory.
fn prompt_output_directory(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<PathBuf, AppError> {
    loop {
        write_to(writer, "Output directory: ");
        let line = read_line(reader)?;
        match validate_output_directory(&line) {
            Ok(dir) => return Ok(dir),
            Err(err) => writeln_to(writer, &format!("  Error: {err}")),
        }
    }
}

/// Prompt for a valid output filename without path traversal.
fn prompt_output_filename(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<String, AppError> {
    loop {
        write_to(writer, "Output filename: ");
        let line = read_line(reader)?;
        match validate_filename(&line) {
            Ok(filename) => return Ok(filename),
            Err(err) => writeln_to(writer, &format!("  Error: {err}")),
        }
    }
}

// ─── I/O Helpers ─────────────────────────────────────────────────────────────

fn write_to(writer: &mut impl Write, s: &str) {
    let _ = write!(writer, "{s}");
    let _ = writer.flush();
}

fn writeln_to(writer: &mut impl Write, s: &str) {
    let _ = writeln!(writer, "{s}");
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Run a full interactive session with simulated input.
    fn session(input: &str) -> Result<Config, AppError> {
        let mut reader = Cursor::new(input.as_bytes().to_vec());
        let mut writer = Vec::new();
        run_session(&mut reader, &mut writer)
    }

    // ── Category parsing ─────────────────────────────────────────────────

    #[test]
    fn parse_categories_valid() {
        assert_eq!(parse_category_selections("1,2,3"), vec![1, 2, 3]);
        assert_eq!(parse_category_selections("4"), vec![4]);
        assert_eq!(parse_category_selections("1, 3, 4"), vec![1, 3, 4]);
    }

    #[test]
    fn parse_categories_ignores_invalid() {
        assert_eq!(parse_category_selections("1,5,2"), vec![1, 2]);
        assert_eq!(parse_category_selections("0,1"), vec![1]);
        assert_eq!(parse_category_selections("abc,2"), vec![2]);
    }

    #[test]
    fn parse_categories_empty_input() {
        assert!(parse_category_selections("").is_empty());
        assert!(parse_category_selections("abc").is_empty());
        assert!(parse_category_selections("0").is_empty());
    }

    #[test]
    fn parse_categories_duplicates_preserved() {
        // build_charset handles deduplication downstream.
        assert_eq!(parse_category_selections("1,1,2"), vec![1, 1, 2]);
    }

    // ── EOF handling ─────────────────────────────────────────────────────

    #[test]
    fn eof_returns_interactive_quit() {
        assert_eq!(session(""), Err(AppError::InteractiveQuit));
    }

    #[test]
    fn eof_mid_flow_returns_interactive_quit() {
        // Select Generate, then hit EOF.
        assert_eq!(session("1\n"), Err(AppError::InteractiveQuit));
    }

    // ── Single password generation ───────────────────────────────────────

    #[test]
    fn interactive_single_password() {
        // Generate → Single → length=20 → charset=lower+upper+digits
        let config = session("1\n1\n20\n1,2,3\n").unwrap();
        assert_eq!(config.mode, Mode::Generate { length: 20 });
        assert_eq!(config.charset.len(), 62);
    }

    // ── Dataset generation ───────────────────────────────────────────────

    #[test]
    fn interactive_dataset() {
        // Generate → Dataset → count=5000 → len=16 → output dir + filename → charset=digits
        let config = session("1\n2\n5000\n16\n/tmp\nout.txt\n3\n").unwrap();
        assert_eq!(
            config.mode,
            Mode::Dataset {
                count: 5000,
                record_length: 16,
                output_path: "/tmp/out.txt".to_string(),
            }
        );
        assert_eq!(config.charset.len(), 10);
    }

    // ── Analyze flow ─────────────────────────────────────────────────────

    #[test]
    fn interactive_analyze_with_rate() {
        // Analyze → min=8 → max=16 → charset=lower+upper → rate=1000
        let config = session("2\n8\n16\n1,2\n1000\n").unwrap();
        assert_eq!(
            config.mode,
            Mode::Analyze {
                min_length: 8,
                max_length: 16,
                rate: Some(1000),
            }
        );
        assert_eq!(config.charset.len(), 52);
    }

    #[test]
    fn interactive_analyze_without_rate() {
        // Analyze → min=4 → max=8 → charset=digits → rate=skip
        let config = session("2\n4\n8\n3\n\n").unwrap();
        assert_eq!(
            config.mode,
            Mode::Analyze {
                min_length: 4,
                max_length: 8,
                rate: None,
            }
        );
        assert_eq!(config.charset.len(), 10);
    }

    // ── Back navigation ──────────────────────────────────────────────────

    #[test]
    fn back_from_generate_submenu() {
        // Generate → Back → Analyze (valid config)
        let config = session("1\n0\n2\n8\n16\n1\n\n").unwrap();
        assert!(matches!(config.mode, Mode::Analyze { .. }));
    }

    #[test]
    fn back_from_charset_in_single_password() {
        // Generate → Single → length=10 → charset=Back → Analyze (valid config)
        let config = session("1\n1\n10\n0\n2\n4\n8\n2\n\n").unwrap();
        assert!(matches!(config.mode, Mode::Analyze { .. }));
    }

    // ── Invalid input re-prompting ───────────────────────────────────────

    #[test]
    fn invalid_main_menu_reprompts() {
        // "5" invalid, "abc" invalid, then valid "1" for Generate
        let config = session("5\nabc\n1\n1\n10\n1\n").unwrap();
        assert_eq!(config.mode, Mode::Generate { length: 10 });
    }

    #[test]
    fn invalid_length_reprompts() {
        // Generate → Single → "0" out-of-range → "abc" NaN → "20" valid
        let config = session("1\n1\n0\nabc\n20\n1,2\n").unwrap();
        assert_eq!(config.mode, Mode::Generate { length: 20 });
    }

    // ── Analyze validation ───────────────────────────────────────────────

    #[test]
    fn analyze_inverted_range_goes_back() {
        // Analyze → min=20, max=10 → error/back → Analyze (valid)
        let config = session("2\n20\n10\n2\n4\n8\n3\n\n").unwrap();
        assert_eq!(
            config.mode,
            Mode::Analyze {
                min_length: 4,
                max_length: 8,
                rate: None,
            }
        );
    }

    #[test]
    fn analyze_zero_rate_goes_back() {
        // Analyze → valid lengths → charset → rate=0 → error/back → Analyze (valid)
        let config = session("2\n8\n16\n1\n0\n2\n4\n8\n3\n100\n").unwrap();
        assert_eq!(
            config.mode,
            Mode::Analyze {
                min_length: 4,
                max_length: 8,
                rate: Some(100),
            }
        );
    }

    // ── Numeric edge cases ───────────────────────────────────────────────

    #[test]
    fn dataset_zero_count_reprompts() {
        // Generate → Dataset → count=0 (invalid) → count=100 → rest valid
        let config = session("1\n2\n0\n100\n8\n/tmp\nx.txt\n1\n").unwrap();
        assert!(matches!(config.mode, Mode::Dataset { count: 100, .. }));
    }

    #[test]
    fn length_at_min_boundary() {
        // Generate → Single → length=1 (minimum) → charset=lowercase
        let config = session("1\n1\n1\n1\n").unwrap();
        assert_eq!(config.mode, Mode::Generate { length: 1 });
    }

    #[test]
    fn length_at_max_boundary() {
        // Generate → Single → length=256 (maximum) → charset=digits
        let config = session("1\n1\n256\n3\n").unwrap();
        assert_eq!(config.mode, Mode::Generate { length: 256 });
    }

    #[test]
    fn length_beyond_max_reprompts() {
        // Generate → Single → "257" out-of-range → "128" valid
        let config = session("1\n1\n257\n128\n1,2,3,4\n").unwrap();
        assert_eq!(config.mode, Mode::Generate { length: 128 });
    }

    #[test]
    fn empty_output_path_reprompts() {
        // Generate → Dataset → count=10 → len=8 → "" empty dir/filename → "/tmp" / "a.txt" → charset
        let config = session("1\n2\n10\n8\n\n/tmp\n\na.txt\n1\n").unwrap();
        assert!(matches!(config.mode, Mode::Dataset { .. }));
    }

    #[test]
    fn back_from_dataset_charset() {
        // Generate → Dataset → count=100 → len=12 → dir + filename → 0 (back) → Calculate keyspace
        let config = session("1\n2\n100\n12\n/tmp\nx.txt\n0\n2\n5\n10\n1\n\n").unwrap();
        assert!(matches!(config.mode, Mode::Analyze { .. }));
    }

    #[test]
    fn back_from_analyze_charset() {
        // Calculate keyspace → min=6 → max=12 → 0 (back) → Generate single password
        let config = session("2\n6\n12\n0\n1\n1\n16\n1,3\n").unwrap();
        assert_eq!(config.mode, Mode::Generate { length: 16 });
    }

    #[test]
    fn negative_inputs_reprompt() {
        // Main menu: -1 -> 1; Submenu: -2 -> 1; Length: -5 -> 20; Charset: 1
        let config = session("-1\n1\n-2\n1\n-5\n20\n1\n").unwrap();
        assert_eq!(config.mode, Mode::Generate { length: 20 });
    }

    #[test]
    fn rate_invalid_input_reprompts() {
        // Calculate → min=4 → max=8 → charset=1 → rate: "abc" (invalid) → rate: 5000
        let config = session("2\n4\n8\n1\nabc\n5000\n").unwrap();
        assert_eq!(
            config.mode,
            Mode::Analyze {
                min_length: 4,
                max_length: 8,
                rate: Some(5000),
            }
        );
    }

    #[test]
    fn all_categories_selected() {
        // Lowercase (26) + Uppercase (26) + Digits (10) + Symbols (32) = 94
        let config = session("1\n1\n10\n1,2,3,4\n").unwrap();
        assert_eq!(config.charset.len(), 94);
    }

    // ── Directory and Filename validation tests ──────────────────────────

    #[test]
    fn test_valid_absolute_directory() {
        let temp_dir = std::env::temp_dir();
        let res = validate_output_directory(&temp_dir.to_string_lossy());
        assert!(res.is_ok());
        let path = res.unwrap();
        assert!(path.is_absolute());
        assert!(path.is_dir());
    }

    #[test]
    fn test_valid_relative_directory() {
        let res = validate_output_directory(".");
        assert!(res.is_ok());
        let path = res.unwrap();
        assert!(path.is_dir());
    }

    #[test]
    fn test_nonexistent_directory() {
        let res = validate_output_directory("/nonexistent_directory_keyspace_12345");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_file_as_directory_rejected() {
        let res = validate_output_directory("Cargo.toml");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not a directory"));
    }

    #[test]
    fn test_empty_directory_rejected() {
        let res = validate_output_directory("");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("cannot be empty"));

        let res = validate_output_directory("   ");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_empty_filename() {
        assert!(validate_filename("").is_err());
        assert!(validate_filename("   ").is_err());
    }

    #[test]
    fn test_invalid_filename() {
        // Traversal and separators
        assert!(validate_filename("..").is_err());
        assert!(validate_filename(".").is_err());
        assert!(validate_filename("../passwords.txt").is_err());
        assert!(validate_filename("foo/bar.txt").is_err());
        assert!(validate_filename("foo\\bar.txt").is_err());
        assert!(validate_filename("/root.txt").is_err());
        assert!(validate_filename("null\0byte.txt").is_err());
    }

    #[test]
    fn test_valid_filenames() {
        assert_eq!(validate_filename("dataset.txt").unwrap(), "dataset.txt");
        assert_eq!(
            validate_filename("wordlist_100k.csv").unwrap(),
            "wordlist_100k.csv"
        );
        assert_eq!(validate_filename(".hidden").unwrap(), ".hidden");
        assert_eq!(validate_filename("passwords").unwrap(), "passwords");
    }

    #[test]
    fn test_successful_output_path_construction() {
        let dir = validate_output_directory(".").unwrap();
        let filename = validate_filename("dataset_test.txt").unwrap();
        let full_path = dir.join(&filename);
        assert_eq!(full_path, Path::new(".").join("dataset_test.txt"));
        assert_eq!(full_path.file_name().unwrap(), "dataset_test.txt");
    }

    #[test]
    fn test_cli_output_compatibility() {
        use crate::cli::Cli;
        let cli = Cli {
            analyze: false,
            generate: false,
            dataset: true,
            min_length: None,
            max_length: None,
            length: None,
            lowercase: true,
            uppercase: false,
            digits: true,
            symbols: false,
            custom: None,
            rate: None,
            count: Some(500),
            record_length: Some(16),
            output: Some("/tmp/legacy_cli_output.txt".to_string()),
        };
        let cfg = Config::from_cli(&cli).unwrap();
        assert_eq!(
            cfg.mode,
            Mode::Dataset {
                count: 500,
                record_length: 16,
                output_path: "/tmp/legacy_cli_output.txt".to_string(),
            }
        );
    }

    #[test]
    fn test_graceful_handling_of_filesystem_errors() {
        let res = check_dir_writable(Path::new("/nonexistent_path_probe_test"));
        assert!(res.is_err());

        let res = check_dir_writable(Path::new("Cargo.toml"));
        assert!(res.is_err());
    }

    #[test]
    fn test_interactive_dataset_reprompts_on_directory_and_filename_errors() {
        // Generate -> Dataset -> count=50 -> len=8 ->
        // dir: "" -> dir: "/nonexistent_dir_9999" -> dir: "/tmp" ->
        // filename: "" -> filename: "../traversal.txt" -> filename: "data.txt" ->
        // charset: 1
        let config = session(
            "1\n2\n50\n8\n\n/nonexistent_dir_9999\n/tmp\n\n../traversal.txt\ndata.txt\n1\n",
        )
        .unwrap();
        assert_eq!(
            config.mode,
            Mode::Dataset {
                count: 50,
                record_length: 8,
                output_path: "/tmp/data.txt".to_string(),
            }
        );
    }
}
