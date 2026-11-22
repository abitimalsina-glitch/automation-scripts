pub mod formatting;

pub use formatting::{format_duration, format_elapsed_time, format_with_commas};

use crate::analysis::entropy::entropy_bits;
use crate::analysis::estimator::estimate_search_time;
use crate::analysis::keyspace::{combinations, total_keyspace};
use crate::benchmark::metrics::DatasetMetrics;
use num_bigint::BigUint;

pub fn display_generation(password: &str, length: usize, duration: std::time::Duration) {
    println!("Password Generation Complete");
    println!();
    println!("Generated Password : {password}");
    println!("Password Length    : {length} characters");
    println!("Generation Time    : {}", format_elapsed_time(duration));
}

pub fn display_dataset_report(metrics: &DatasetMetrics, output_path: &str) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                   DATASET GENERATION COMPLETE                    ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Output File      : {output_path}");
    println!(
        "  Total Records    : {}",
        format_with_commas(&BigUint::from(metrics.total_records))
    );
    println!(
        "  Total Size       : {:.2} KB ({} bytes)",
        metrics.total_bytes as f64 / 1024.0,
        format_with_commas(&BigUint::from(metrics.total_bytes))
    );
    println!(
        "  Avg Record Size  : {:.2} bytes",
        metrics.avg_bytes_per_record()
    );
    println!(
        "  Execution Time   : {}",
        format_elapsed_time(metrics.elapsed)
    );
    println!("  Records/Second   : {:.2}", metrics.records_per_second());
    println!("  Throughput       : {:.4} MB/s", metrics.throughput_mbps());
    println!();
}

pub fn display_analysis(charset: &[char], min_len: usize, max_len: usize, rate: Option<u64>) {
    let charset_size = charset.len();
    let charset_display: String = charset.iter().collect();

    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                       KEYSPACE ANALYSIS                          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Character set size : {charset_size}");
    println!("  Characters         : {charset_display}");
    println!("  Length range        : {min_len} to {max_len}");
    println!();

    println!(
        "  {:>8}  {:>44}  {:>16}",
        "Length", "Combinations", "Entropy (bits)"
    );
    println!("  {}", "─".repeat(72));

    for len in min_len..=max_len {
        let combos = combinations(charset_size, len);
        let entropy = entropy_bits(charset_size, len);
        println!(
            "  {:>8}  {:>44}  {:>14.2}",
            len,
            format_with_commas(&combos),
            entropy,
        );
    }

    let total = total_keyspace(charset_size, min_len, max_len);
    println!("  {}", "─".repeat(72));
    println!("  {:>8}  {:>44}", "TOTAL", format_with_commas(&total));

    if let Some(rate) = rate {
        println!();
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                   SEARCH TIME ESTIMATION                         ║");
        println!("╚══════════════════════════════════════════════════════════════════╝");
        println!();
        println!(
            "  Rate: {} guesses/second",
            format_with_commas(&BigUint::from(rate))
        );
        println!();
        println!("  Assumptions:");
        println!("    • Constant search rate with no computational overhead");
        println!("    • Uniform random key distribution (all keys equally likely)");
        println!("    • No early-termination optimizations");
        println!("    • The supplied rate is hypothetical, not measured hardware performance");
        println!();
        println!("  Formulas:");
        println!("    • Best case    = 1 / rate");
        println!("    • Average case = total_keyspace / (2 × rate)");
        println!("    • Worst case   = total_keyspace / rate");
        println!();

        let times = estimate_search_time(&total, rate);

        println!(
            "  Best case (1st guess):     {}",
            format_duration(&times.best)
        );
        println!(
            "  Average case (50%):        {}",
            format_duration(&times.average)
        );
        println!(
            "  Worst case (exhaustive):   {}",
            format_duration(&times.worst)
        );
    }

    println!();
}
