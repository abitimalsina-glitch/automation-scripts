use clap::Parser;

#[derive(Parser)]
#[command(
    name = "keyspace-tool",
    about = "Analyze password keyspaces and generate secure random passwords."
)]
pub struct Cli {
    /// Analyze the keyspace for the given configuration
    #[arg(long, group = "mode")]
    pub analyze: bool,

    /// Generate one secure random password
    #[arg(long, group = "mode")]
    pub generate: bool,

    /// Minimum password length (analyze mode)
    #[arg(long)]
    pub min_length: Option<usize>,

    /// Maximum password length (analyze mode)
    #[arg(long)]
    pub max_length: Option<usize>,

    /// Exact password length (generate mode)
    #[arg(long)]
    pub length: Option<usize>,

    /// Include lowercase letters (a-z)
    #[arg(long)]
    pub lowercase: bool,

    /// Include uppercase letters (A-Z)
    #[arg(long)]
    pub uppercase: bool,

    /// Include digits (0-9)
    #[arg(long)]
    pub digits: bool,

    /// Include special characters
    #[arg(long)]
    pub symbols: bool,

    /// Custom ASCII characters to include
    #[arg(long)]
    pub custom: Option<String>,

    /// Hypothetical guesses per second for time estimation (analyze mode)
    #[arg(long)]
    pub rate: Option<u64>,
}
