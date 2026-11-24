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
    writeln_to(writer, "║                WORDLIST GENERATOR              ║");
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
