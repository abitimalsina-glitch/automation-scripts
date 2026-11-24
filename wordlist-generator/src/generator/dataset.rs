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
