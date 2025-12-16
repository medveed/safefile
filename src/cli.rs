use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Encrypt files with AES256-GCM and split key with SSS"
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Encrypt a file and split the key into shares (default 5 shares, threshold 3)")]
    Encrypt {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long, default_value_t = 5)]
        shares: usize,
        #[arg(short, long, default_value_t = 3)]
        threshold: usize,
        #[arg(short, long)]
        outdir: Option<PathBuf>,
        #[arg(short = 'l', long)]
        label: Option<String>,
    },
    #[command(about = "Decrypt a safe file using provided share files")]
    Decrypt {
        input: PathBuf,
        output: PathBuf,
        shares: Vec<PathBuf>,
    },
    #[command(about = "Inspect a safe file and display its metadata")]
    Info {
        input: PathBuf,
    },
}
