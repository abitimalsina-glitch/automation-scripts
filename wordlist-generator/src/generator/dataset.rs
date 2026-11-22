use crate::benchmark::metrics::DatasetMetrics;
use crate::error::AppError;
use crate::generator::password::generate_password;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

/// Progress reporting interval: report every this many records or 1% of total, whichever is larger.
const MIN_PROGRESS_INTERVAL: u64 = 10_000;

/// Generate a synthetic password dataset and stream it to a file.
///
/// Each record is one password per line, generated using the system CSPRNG.
/// Uses `BufWriter` for memory-efficient streaming — the full dataset is never
/// accumulated in memory.
///
/// Progress is reported to stderr periodically for large datasets.
pub fn generate_dataset(
    charset: &[char],
    record_length: usize,
    count: u64,
    output_path: &str,
) -> Result<DatasetMetrics, AppError> {
    let file = File::create(output_path).map_err(|e| AppError::FileError {
        path: output_path.to_string(),
        message: e.to_string(),
    })?;
    let mut writer = BufWriter::new(file);

    let progress_interval = std::cmp::max(MIN_PROGRESS_INTERVAL, count / 100);

    let start = Instant::now();
    let mut total_bytes: u64 = 0;

    for i in 0..count {
        let password = generate_password(charset, record_length);
        let line = format!("{password}\n");
        let bytes = line.as_bytes();
        total_bytes += bytes.len() as u64;
        writer.write_all(bytes).map_err(|e| AppError::FileError {
            path: output_path.to_string(),
            message: e.to_string(),
        })?;

        // Progress reporting to stderr (not per-record, periodic)
        if count > MIN_PROGRESS_INTERVAL && (i + 1) % progress_interval == 0 {
            let pct = ((i + 1) as f64 / count as f64) * 100.0;
            eprint!("\r  Progress: {}/{} records ({:.1}%)", i + 1, count, pct);
        }
    }

    writer.flush().map_err(|e| AppError::FileError {
        path: output_path.to_string(),
        message: e.to_string(),
    })?;

    let elapsed = start.elapsed();

    // Clear the progress line if we printed any
    if count > MIN_PROGRESS_INTERVAL {
        eprint!("\r{}\r", " ".repeat(60));
    }

    Ok(DatasetMetrics {
        total_records: count,
        total_bytes,
        elapsed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::build_charset;
    use std::fs;
    use std::path::PathBuf;

    /// Create a unique temporary file path for testing.
    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("keyspace_test_{name}_{}", std::process::id()))
    }

    /// Clean up a test file, ignoring errors.
    fn cleanup(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn dataset_produces_correct_record_count() {
        let path = temp_path("count");
        let charset = build_charset(true, false, false, false, None);
        let result = generate_dataset(&charset, 10, 100, path.to_str().unwrap());
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.total_records, 100);

        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 100);
        cleanup(&path);
    }

    #[test]
    fn dataset_records_have_correct_length() {
        let path = temp_path("length");
        let charset = build_charset(true, true, true, false, None);
        generate_dataset(&charset, 20, 50, path.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        for line in content.lines() {
            assert_eq!(line.chars().count(), 20);
        }
        cleanup(&path);
    }

    #[test]
    fn dataset_records_use_only_charset_chars() {
        let path = temp_path("charset");
        let charset = build_charset(false, false, true, false, None); // digits only
        generate_dataset(&charset, 12, 50, path.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        for line in content.lines() {
            for ch in line.chars() {
                assert!(
                    charset.contains(&ch),
                    "unexpected char '{ch}' not in charset"
                );
            }
        }
        cleanup(&path);
    }

    #[test]
    fn dataset_one_record_per_line() {
        let path = temp_path("format");
        let charset = build_charset(true, false, false, false, None);
        generate_dataset(&charset, 8, 25, path.to_str().unwrap()).unwrap();

        let raw = fs::read_to_string(&path).unwrap();
        // The file should end with a newline
        assert!(raw.ends_with('\n'));
        // Number of newlines == number of records
        assert_eq!(raw.chars().filter(|c| *c == '\n').count(), 25);
        cleanup(&path);
    }

    #[test]
    fn dataset_total_bytes_is_accurate() {
        let path = temp_path("bytes");
        let charset = build_charset(true, false, false, false, None);
        let record_len = 10;
        let count = 50;
        let metrics =
            generate_dataset(&charset, record_len, count, path.to_str().unwrap()).unwrap();

        let file_size = fs::metadata(&path).unwrap().len();
        assert_eq!(metrics.total_bytes, file_size);
        // Each line is record_len chars + 1 newline = 11 bytes (ASCII)
        assert_eq!(metrics.total_bytes, (record_len as u64 + 1) * count);
        cleanup(&path);
    }

    #[test]
    fn dataset_single_record() {
        let path = temp_path("single");
        let charset = build_charset(true, true, true, true, None);
        let metrics = generate_dataset(&charset, 16, 1, path.to_str().unwrap()).unwrap();

        assert_eq!(metrics.total_records, 1);
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.lines().count(), 1);
        assert_eq!(content.lines().next().unwrap().chars().count(), 16);
        cleanup(&path);
    }

    #[test]
    fn dataset_large_streaming_no_oom() {
        // Generate 10,000+ records to verify streaming works without OOM.
        let path = temp_path("large");
        let charset = build_charset(true, true, true, false, None);
        let count = 10_000;
        let metrics = generate_dataset(&charset, 16, count, path.to_str().unwrap()).unwrap();

        assert_eq!(metrics.total_records, count);
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.lines().count(), count as usize);
        cleanup(&path);
    }

    #[test]
    fn dataset_elapsed_time_is_non_negative() {
        let path = temp_path("timing");
        let charset = build_charset(true, false, false, false, None);
        let metrics = generate_dataset(&charset, 8, 100, path.to_str().unwrap()).unwrap();
        assert!(metrics.elapsed >= std::time::Duration::ZERO);
        cleanup(&path);
    }

    #[test]
    fn dataset_invalid_output_path() {
        let charset = build_charset(true, false, false, false, None);
        let result = generate_dataset(&charset, 10, 10, "/nonexistent_dir/impossible/test.txt");
        assert!(result.is_err());
        if let Err(AppError::FileError { path, message }) = result {
            assert_eq!(path, "/nonexistent_dir/impossible/test.txt");
            assert!(!message.is_empty());
        } else {
            panic!("expected FileError");
        }
    }
}
