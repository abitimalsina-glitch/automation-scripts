mod analysis;
mod benchmark;
mod cli;
mod config;
mod error;
mod generator;
mod interactive;
mod output;

use clap::Parser;
use config::{Config, Mode};
use error::AppError;

fn run() -> Result<(), AppError> {
    let config = if std::env::args().len() == 1 {
        interactive::run_interactive()?
    } else {
        let cli = cli::Cli::parse();
        Config::from_cli(&cli)?
    };

    match config.mode {
        Mode::Analyze {
            min_length,
            max_length,
            rate,
        } => {
            output::display_analysis(&config.charset, min_length, max_length, rate);
        }
        Mode::Generate { length } => {
            let (password, elapsed) =
                generator::password::generate_password_timed(&config.charset, length);
            output::display_generation(&password, length, elapsed);
        }
        Mode::Dataset {
            count,
            record_length,
            output_path,
        } => {
            let metrics = generator::dataset::generate_dataset(
                &config.charset,
                record_length,
                count,
                &output_path,
            )?;
            output::display_dataset_report(&metrics, &output_path);
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        if matches!(err, AppError::InteractiveQuit) {
            eprintln!("\nGoodbye!");
        } else {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    }
}
