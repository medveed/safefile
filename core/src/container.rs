//! Container definitions for on-disk structures.

use serde::{Deserialize, Serialize};

/// Safefile header.
///
/// - `version` identifies the format version.
/// - `timestamp` is UNIX seconds when the file was created.
/// - `label` is an optional user label.
/// - `nonce` is a 12-byte AES nonce used for encryption.
/// - `ciphertext_len` is the size of the ciphertext in bytes.
#[derive(Serialize, Deserialize)]
pub struct SafeHeader {
    pub version: u8,
    pub timestamp: u64,
    pub label: String,
    pub nonce: [u8; 12],
    pub ciphertext_len: u64,
}

/// Single key share written to disk.
///
/// - `version` identifies the format version.
/// - `timestamp` is UNIX seconds when the file was created.
/// - `label` is an optional user label.
/// - `share` contains the raw share bytes.
/// - `checksum` is a SHA-256 of the `share` for integrity verification.
#[derive(Serialize, Deserialize)]
pub struct ShareFile {
    pub version: u8,
    pub timestamp: u64,
    pub label: String,
    pub share: Vec<u8>,
    pub checksum: [u8; 32],
}
