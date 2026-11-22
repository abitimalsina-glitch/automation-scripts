mod analysis;
mod cli;
mod config;
mod generator;
mod utils;

use clap::Parser;
use num_bigint::BigUint;
use std::process;

use analysis::entropy::entropy_bits;
use analysis::estimator::estimate_search_time;
use analysis::keyspace::{combinations, total_keyspace};
use config::{build_charset, validate_length};
use generator::password::generate_password_timed;
use utils::formatting::{format_duration, format_elapsed_time, format_with_commas};

// ─── Display ─────────────────────────────────────────────────────────────────

fn display_generation(password: &str, length: usize, duration: std::time::Duration) {
    println!("Password Generation Complete");
    println!();
    println!("Generated Password : {password}");
    println!("Password Length    : {length} characters");
    println!("Generation Time    : {}", format_elapsed_time(duration));
}

fn display_analysis(charset: &[char], min_len: usize, max_len: usize, rate: Option<u64>) {
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

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    let cli = cli::Cli::parse();

    if !cli.analyze && !cli.generate {
        eprintln!("Error: specify either --analyze or --generate.");
        eprintln!("Run with --help for usage information.");
        process::exit(1);
    }

    let charset = build_charset(
        cli.lowercase,
        cli.uppercase,
        cli.digits,
        cli.symbols,
        cli.custom.as_deref(),
    );

    if charset.is_empty() {
        eprintln!(
            "Error: no characters selected. \
             Use --lowercase, --uppercase, --digits, --symbols, or --custom."
        );
        process::exit(1);
    }

    if cli.analyze {
        let min_len = match cli.min_length {
            Some(v) => v,
            None => {
                eprintln!("Error: --min-length is required for --analyze.");
                process::exit(1);
            }
        };
        let max_len = match cli.max_length {
            Some(v) => v,
            None => {
                eprintln!("Error: --max-length is required for --analyze.");
                process::exit(1);
            }
        };

        if let Err(e) = validate_length(min_len, "--min-length") {
            eprintln!("Error: {e}");
            process::exit(1);
        }
        if let Err(e) = validate_length(max_len, "--max-length") {
            eprintln!("Error: {e}");
            process::exit(1);
        }
        if min_len > max_len {
            eprintln!("Error: --min-length ({min_len}) must not exceed --max-length ({max_len}).");
            process::exit(1);
        }

        if let Some(rate) = cli.rate {
            if rate == 0 {
                eprintln!("Error: --rate must be greater than 0.");
                process::exit(1);
            }
        }

        display_analysis(&charset, min_len, max_len, cli.rate);
    } else if cli.generate {
        let length = match cli.length {
            Some(v) => v,
            None => {
                eprintln!("Error: --length is required for --generate.");
                process::exit(1);
            }
        };

        if let Err(e) = validate_length(length, "--length") {
            eprintln!("Error: {e}");
            process::exit(1);
        }

        let (password, elapsed) = generate_password_timed(&charset, length);
        display_generation(&password, length, elapsed);
    }
}
