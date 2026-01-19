//! High-level operations that compose encryption and secret sharing.

use crate::consts;
use crate::container::SafeHeader;
use crate::crypto;
use crate::error::Error;
use crate::format;
use crate::format::SafeInfo;
use crate::shamir;
use crate::utils;
use aes_gcm::aead::OsRng;
use aes_gcm::aead::rand_core::RngCore;
use std::path::Path;
use std::time::SystemTime;
use zeroize::Zeroize;

pub struct EncryptResult {
    #[allow(unused)]
    pub safe_file: String,
    pub share_files: Vec<String>,
    pub info: SafeInfo,
}

pub struct DecryptResult {
    pub output_file: String,
    pub info: SafeInfo,
}

/// Does the whole process of encyprion and splitting.
/// 
/// - Generates a random 256-bit key
/// - Stream-encrypts an input file with AES-256-GCM
/// - Splits the key using SSS and writes shares to disk
pub fn encrypt_and_split<F>(
    input: &Path,
    output: &Path,
    outdir: &Path,
    shares: u8,
    threshold: u8,
    label: Option<&str>,
    progress_callback: F,
) -> Result<EncryptResult, Error>
where
    F: FnMut(u64, u64),
{
    let mut io_timer = utils::Timer::new();
    let mut crypto_timer = utils::Timer::new();

    io_timer.start();
    let metadata = std::fs::metadata(input).map_err(|e| Error::Io {
        path: Some(input.to_path_buf()),
        source: e,
    })?;
    let pt_len = metadata.len();
    io_timer.stop();

    crypto_timer.start();

    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    crypto_timer.stop();

    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| Error::InternalError {
            details: format!("time error: {}", e),
        })?
        .as_secs();
    let lab = label.unwrap_or("").to_string();
    let header = SafeHeader {
        version: consts::VERSION,
        timestamp: ts,
        label: lab.clone(),
        nonce,
        ciphertext_len: pt_len,
    };

    io_timer.start();
    crypto::encrypt_stream(input, output, key, nonce, header, progress_callback)?;
    io_timer.stop();

    crypto_timer.start();
    let share_paths = shamir::create_shares(&key, threshold, shares, outdir, &lab, ts)?;

    // Zero the key and nonce
    key.zeroize();
    nonce.zeroize();
    crypto_timer.stop();

    let info = format::inspect_safe_from_path(output)?;

    Ok(EncryptResult {
        safe_file: output.to_string_lossy().into_owned(),
        share_files: share_paths,
        info: format::SafeInfo {
            version: info.version,
            timestamp: info.timestamp,
            label: info.label,
            ciphertext_len: info.ciphertext_len,
        },
    })
}


/// Does the whole process of recunstruction and decryption.
/// 
/// - Reads key shares
/// - Tries to reconstruct the key
/// - Stream-decrypts the safefile
pub fn decrypt_and_reconstruct<F>(
    safe_path: &Path,
    output: &Path,
    share_paths: &[&Path],
    progress_callback: F,
) -> Result<DecryptResult, Error>
where
    F: FnMut(u64, u64),
{
    let mut timer = utils::Timer::new();
    timer.start();

    let mut key_vec = shamir::reconstruct_key(share_paths)?;
    let mut key = [0u8; 32];
    if key_vec.len() != 32 {
        return Err(Error::InternalError {
            details: "reconstructed key wrong length".into(),
        });
    }
    key.copy_from_slice(&key_vec);

    key_vec.zeroize();

    crypto::decrypt_stream(safe_path, output, key, progress_callback)?;

    // Zero key after decryption
    key.zeroize();

    let info = format::inspect_safe_from_path(safe_path)?;

    Ok(DecryptResult {
        output_file: output.to_string_lossy().into_owned(),
        info: format::SafeInfo {
            version: info.version,
            timestamp: info.timestamp,
            label: info.label,
            ciphertext_len: info.ciphertext_len,
        },
    })
}
