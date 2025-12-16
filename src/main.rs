mod cli;
mod consts;
mod container;
mod crypto;
mod error;
mod utils;

use chrono::{TimeZone, Utc};
use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use error::Error;
use std::path::{Path, PathBuf};

fn main() {
    let res = run();
    if let Err(e) = res {
        eprintln!("{} {}", "Error:".red().bold(), format!("{}", e).red());
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Encrypt {
            input,
            output,
            shares,
            threshold,
            outdir,
            label,
        } => {
            println!(
                "{} {} -> {}",
                "Encrypting:".green().bold(),
                input.display(),
                output.display()
            );

            let shares_u8 = u8::try_from(shares).map_err(|_| Error::InvalidArgument {
                details: format!("shares must be in range [1;255] (got {})", shares),
            })?;
            let threshold_u8 = u8::try_from(threshold).map_err(|_| Error::InvalidArgument {
                details: format!("threshold must be in range [1;255] (got {})", threshold),
            })?;

            let outdir_path: PathBuf = match outdir {
                Some(p) => p,
                None => output
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from(".")),
            };

            let result = crypto::encrypt_and_split(
                &input,
                &output,
                &outdir_path,
                shares_u8,
                threshold_u8,
                label.as_deref(),
            )?;

            let when = Utc
                .timestamp_opt(result.info.timestamp as i64, 0)
                .single()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| result.info.timestamp.to_string());

            table_row!("Version:", result.info.version);
            table_row!("All/Min: ", format!("{} / {}", shares, threshold));
            table_row!("Timestamp:", when);
            if !result.info.label.is_empty() {
                table_row!("Label:", result.info.label);
            }
            table_row!(
                "Size:",
                utils::bytes_to_human_readable(result.info.ciphertext_len)
            );
            table_row!(
                "Duration:",
                format!(
                    "{} (IO: {}, Crypto: {})",
                    utils::us_to_human_readable(result.time.io_us + result.time.crypto_us),
                    utils::us_to_human_readable(result.time.io_us),
                    utils::us_to_human_readable(result.time.crypto_us),
                )
            );
            table_row!(
                "Avg speed:",
                utils::bytes_to_human_readable(
                    (result.info.ciphertext_len as u128 * 1_000_000 / result.time.crypto_us)
                        as usize
                ) + "/s"
            );
            println!(
                "{} {} share files to '{}'",
                "Wrote".green(),
                result.share_files.len(),
                outdir_path.display()
            );
        }
        Commands::Decrypt {
            input,
            output,
            shares,
        } => {
            println!(
                "{} {} -> {}",
                "Decrypting:".green().bold(),
                input.display(),
                output.display()
            );

            let share_paths: Vec<&Path> = shares.iter().map(|p| p.as_path()).collect();
            let written = crypto::decrypt_and_reconstruct(&input, &output, &share_paths)?;
            println!("{} {}", "Recovered:".green(), written.output_file);
            table_row!(
                "Size:",
                utils::bytes_to_human_readable(written.info.ciphertext_len)
            );
            table_row!(
                "Duration:",
                format!(
                    "{} (IO: {}, Crypto: {})",
                    utils::us_to_human_readable(written.time.io_us + written.time.crypto_us),
                    utils::us_to_human_readable(written.time.io_us),
                    utils::us_to_human_readable(written.time.crypto_us),
                )
            );
            table_row!(
                "Avg speed:",
                utils::bytes_to_human_readable(
                    (written.info.ciphertext_len as u128 * 1_000_000 / written.time.crypto_us)
                        as usize
                ) + "/s"
            );
        }
        Commands::Info { input } => {
            println!("{} {}", "Inspecting:".green().bold(), input.display());
            let info = crypto::inspect_safe_from_path(&input)?;
            let when = Utc
                .timestamp_opt(info.timestamp as i64, 0)
                .single()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| info.timestamp.to_string());
            println!("{}", "Info:".green().bold());
            table_row!("Version:", info.version);
            table_row!("Timestamp:", when);
            if !info.label.is_empty() {
                table_row!("Label:", info.label);
            }

            table_row!("Size:", utils::bytes_to_human_readable(info.ciphertext_len));
        }
    }

    Ok(())
}
