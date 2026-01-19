//! Shamir Secret Sharing helpers and share file handling.

use crate::container::ShareFile;
use crate::error::Error;
use sha2::{Digest, Sha256};
use sss_rs::prelude::share;
use std::fs;
use std::path::{Path};
use zeroize::Zeroize;

/// Create SSS shares for `key` and save them to `outdir`.
///
/// The files are named `share_001.bin`, `share_002.bin`, ...
pub fn create_shares(
    key: &[u8],
    threshold: u8,
    shares: u8,
    outdir: &Path,
    label: &str,
    timestamp: u64,
) -> Result<Vec<String>, Error> {
    let mut shares_vec = share(key, threshold, shares, true).map_err(|e| match e {
        sss_rs::wrapped_sharing::Error::IOError(io_err) => io_err.into(),
        sss_rs::wrapped_sharing::Error::OtherSharingError(basic) => match basic {
            sss_rs::basic_sharing::Error::UnreconstructableSecret(required, provided) => {
                Error::NotEnoughShares { required, provided }
            },
            _ => Error::SharingFailed { details: format!("{}", basic) },
        },
        sss_rs::wrapped_sharing::Error::VerificationFailure(_, _) => {
            Error::ShareVerificationFailed { details: "share verification failed".into() }
        },
        _ => Error::SharingFailed { details: format!("sharing failed: {}", e) },
    })?;

    let mut paths = Vec::new();
    for (i, s) in shares_vec.iter().enumerate() {
        let mut hasher = Sha256::new();
        hasher.update(s);
        let digest = hasher.finalize();
        let mut checksum = [0u8; 32];
        checksum.copy_from_slice(&digest);

        let sf = ShareFile {
            version: 1,
            timestamp,
            label: label.to_string(),
            share: s.clone(),
            checksum,
        };

        let data = bincode::serde::encode_to_vec(&sf, bincode::config::standard())?;
        let filename = outdir.join(format!("share_{:03}.bin", i + 1));
        fs::write(&filename, data)?;
        paths.push(filename.to_string_lossy().into_owned());
    }

    // Zero shares in memory after writing to disk
    for s in shares_vec.iter_mut() {
        s.zeroize();
    }

    Ok(paths)
}

/// Reconstruct the original key from a set of share file paths.
///
/// The function will validate each share's checksum.
pub fn reconstruct_key(share_paths: &[&Path]) -> Result<Vec<u8>, Error> {
    let mut shares_buf = Vec::new();
    for p in share_paths {
        let raw = fs::read(p)?;
        let (sfile, _): (ShareFile, usize) = bincode::serde::decode_from_slice(&raw, bincode::config::standard())?;

        let mut hasher = Sha256::new();
        hasher.update(&sfile.share);
        let digest = hasher.finalize();
        if digest[..] != sfile.checksum[..] {
            return Err(Error::ShareChecksumMismatch { path: p.to_path_buf() });
        }
        shares_buf.push(sfile.share);
    }

    let key = sss_rs::prelude::reconstruct(&shares_buf, true).map_err(|e| Error::InternalError { details: format!("reconstruct failed: {}", e) })?;

    // Zero share buffers
    for s in shares_buf.iter_mut() {
        s.zeroize();
    }

    Ok(key)
}
