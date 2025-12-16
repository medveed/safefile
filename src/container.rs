use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SafeHeader {
    pub version: u8,
    pub timestamp: u64,
    pub label: String,
    pub nonce: [u8; 12],
    pub ciphertext_len: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ShareFile {
    pub version: u8,
    pub timestamp: u64,
    pub label: String,
    pub share: Vec<u8>,
    pub checksum: [u8; 32],
}
