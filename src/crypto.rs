use crate::container::{SafeHeader, ShareFile};
use crate::error::Error;
use crate::{consts, utils};
use aes_gcm::aead::{AeadInPlace, KeyInit, OsRng, Tag, rand_core::RngCore};
use aes_gcm::{Aes256Gcm, Nonce};
use sha2::{Digest, Sha256};
use sss_rs::prelude::{reconstruct, share};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct TimeMetrics {
    pub io_us: u128,
    pub crypto_us: u128,
}

pub struct EncryptResult {
    #[allow(unused)]
    pub safe_file: String,
    pub share_files: Vec<String>,
    pub time: TimeMetrics,
    pub info: SafeInfo,
}

pub struct DecryptResult {
    pub output_file: String,
    pub time: TimeMetrics,
    pub info: SafeInfo,
}

pub struct SafeInfo {
    pub version: u8,
    pub timestamp: u64,
    pub label: String,
    pub ciphertext_len: usize,
}

pub fn encrypt_and_split(
    input: &Path,
    output: &Path,
    outdir: &Path,
    shares: u8,
    threshold: u8,
    label: Option<&str>,
) -> Result<EncryptResult, Error> {
    let mut io_chrono = utils::Timer::new();
    let mut crypto_chrono = utils::Timer::new();

    io_chrono.start();
    let mut buffer = fs::read(input)?;
    io_chrono.stop();

    crypto_chrono.start();
    let key = Aes256Gcm::generate_key(OsRng);
    let cipher = Aes256Gcm::new(&key);
    let mut nonce = [0u8; 12];
    OsRng
        .try_fill_bytes(&mut nonce)
        .map_err(|e| Error::InternalError {
            details: format!("RNG error: {}", e),
        })?;
    let nonce_ref = Nonce::from_slice(&nonce);

    let tag = cipher
        .encrypt_in_place_detached(nonce_ref, b"", &mut buffer)
        .map_err(|_e| Error::EncryptionFailed)?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| Error::InternalError {
            details: format!("system time error: {}", e),
        })?
        .as_secs();
    let lab = label.unwrap_or("").to_string();

    let header = SafeHeader {
        version: 1,
        nonce,
        timestamp: ts,
        label: lab.clone(),
        ciphertext_len: buffer.len(),
    };

    let header_bytes = bincode::serde::encode_to_vec(&header, bincode::config::standard())?;

    crypto_chrono.stop();

    io_chrono.start();
    let mut f = File::create(output)?;
    f.write_all(b"SFIL")?;
    f.write_all(&header_bytes)?;
    f.write_all(&buffer)?;
    f.write_all(tag.as_slice())?;
    f.flush()?;

    let key_bytes = key.as_slice().to_vec();
    let shares_vec =
        share(&key_bytes, threshold, shares, true).map_err(|e| Error::SharingFailed {
            details: e.to_string(),
        })?;

    let mut share_files = Vec::new();
    for (i, s) in shares_vec.iter().enumerate() {
        let mut hasher = Sha256::new();
        hasher.update(s);
        let digest = hasher.finalize();
        let mut checksum = [0u8; 32];
        checksum.copy_from_slice(&digest);

        let share_file = ShareFile {
            version: 1,
            timestamp: ts,
            label: lab.clone(),
            share: s.clone(),
            checksum,
        };
        let data = bincode::serde::encode_to_vec(&share_file, bincode::config::standard())?;
        let filename = outdir.join(format!("share_{:03}.bin", i + 1));
        fs::write(&filename, data)?;
        share_files.push(filename.to_string_lossy().into_owned());
    }
    io_chrono.stop();

    Ok(EncryptResult {
        safe_file: output.to_string_lossy().into_owned(),
        share_files,
        time: TimeMetrics {
            io_us: io_chrono.duration_us,
            crypto_us: crypto_chrono.duration_us,
        },
        info: SafeInfo {
            version: header.version,
            timestamp: header.timestamp,
            label: header.label,
            ciphertext_len: header.ciphertext_len,
        },
    })
}

pub fn decrypt_and_reconstruct(
    safe_path: &Path,
    output: &Path,
    share_paths: &[&Path],
) -> Result<DecryptResult, Error> {
    let mut io_chrono = utils::Timer::new();
    let mut crypto_chrono = utils::Timer::new();

    io_chrono.start();

    let file_info = inspect_safe_from_path(safe_path)?;
    if file_info.version != consts::VERSION {
        return Err(Error::UnsupportedVersion {
            path: safe_path.to_path_buf(),
            version: file_info.version,
        });
    }

    let mut data = fs::read(safe_path)?;
    if data.len() < 4 || &data[0..4] != b"SFIL" {
        return Err(Error::InvalidMagic {
            path: safe_path.to_path_buf(),
        });
    }

    let (header, hdr_len): (SafeHeader, usize) =
        bincode::serde::decode_from_slice(&data[4..], bincode::config::standard()).map_err(
            |e| Error::InvalidFormat {
                path: safe_path.to_path_buf(),
                details: format!("header decode failed: {}", e),
            },
        )?;

    if header.version != consts::VERSION {
        return Err(Error::UnsupportedVersion {
            path: safe_path.to_path_buf(),
            version: header.version,
        });
    }

    let hdr_end = 4 + hdr_len;
    let ct_start = hdr_end;
    let ct_end = ct_start + header.ciphertext_len;
    let tag_start = ct_end;
    let tag_end = tag_start + 16;
    if data.len() < tag_end {
        return Err(Error::IncompleteFile {
            path: safe_path.to_path_buf(),
        });
    }

    let mut shares_buf = Vec::new();
    for p in share_paths {
        let raw = fs::read(p)?;
        let (sfile, _): (ShareFile, usize) =
            bincode::serde::decode_from_slice(&raw, bincode::config::standard()).map_err(|_e| {
                Error::ShareCorrupted {
                    path: p.to_path_buf(),
                }
            })?;
        let mut hasher = Sha256::new();
        hasher.update(&sfile.share);
        let digest = hasher.finalize();
        if digest[..] != sfile.checksum[..] {
            return Err(Error::ShareChecksumMismatch {
                path: p.to_path_buf(),
            });
        }
        shares_buf.push(sfile.share);
    }
    io_chrono.stop();

    crypto_chrono.start();
    let key = reconstruct(&shares_buf, true).map_err(|_e| match _e {
        sss_rs::wrapped_sharing::Error::IOError(ioerr) => ioerr.into(),
        sss_rs::wrapped_sharing::Error::VerificationFailure(a, b) => {
            Error::ShareVerificationFailed {
                details: (format!("Verification failed: {} vs {}", a, b)),
            }
        }
        sss_rs::wrapped_sharing::Error::OtherSharingError(basic) => match basic {
            sss_rs::basic_sharing::Error::UnreconstructableSecret(required, provided) => {
                Error::NotEnoughShares { provided, required }
            }
            sss_rs::basic_sharing::Error::InvalidNumberOfShares { .. } => {
                Error::OtherShareReconstructionError {
                    details: "Invalid number of shares".to_string(),
                }
            }
        },
        _ => Error::SharingFailed {
            details: _e.to_string(),
        },
    })?;
    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(&header.nonce);

    {
        let mut tag_arr = [0u8; 16];
        tag_arr.copy_from_slice(&data[tag_start..tag_end]);
        let tag = Tag::<Aes256Gcm>::from_slice(&tag_arr);

        let ct_slice = &mut data[ct_start..ct_end];
        cipher
            .decrypt_in_place_detached(nonce, b"", ct_slice, tag)
            .map_err(|_e| Error::InvalidAuthenticationTag)?;
    }
    crypto_chrono.stop();

    io_chrono.start();
    fs::write(output, &data[ct_start..ct_end]).map_err(|e| Error::Io {
        path: Some(output.to_path_buf()),
        source: e,
    })?;
    io_chrono.stop();

    Ok(DecryptResult {
        output_file: output.to_string_lossy().into_owned(),
        time: TimeMetrics {
            io_us: io_chrono.duration_us,
            crypto_us: crypto_chrono.duration_us,
        },
        info: SafeInfo {
            version: header.version,
            timestamp: header.timestamp,
            label: header.label,
            ciphertext_len: header.ciphertext_len,
        },
    })
}

pub fn inspect_safe_from_path(safe_path: &Path) -> Result<SafeInfo, Error> {
    let mut f = File::open(safe_path).map_err(|e| Error::Io {
        path: Some(safe_path.to_path_buf()),
        source: e,
    })?;

    let mut magic = [0u8; 4];
    f.read_exact(&mut magic).map_err(|e| Error::Io {
        path: Some(safe_path.to_path_buf()),
        source: e,
    })?;
    if &magic != b"SFIL" {
        return Err(Error::InvalidMagic {
            path: safe_path.to_path_buf(),
        });
    }

    let mut buf = Vec::new();

    loop {
        let mut chunk = vec![0u8; 1024];
        let n = f.read(&mut chunk).map_err(|e| Error::Io {
            path: Some(safe_path.to_path_buf()),
            source: e,
        })?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);

        match bincode::serde::decode_from_slice::<SafeHeader, _>(&buf, bincode::config::standard())
        {
            Ok((header, _len)) => {
                return Ok(SafeInfo {
                    version: header.version,
                    timestamp: header.timestamp,
                    label: header.label,
                    ciphertext_len: header.ciphertext_len,
                });
            }
            Err(e) => {
                if buf.len() >= 64 * 1024 {
                    return Err(Error::InvalidFormat {
                        path: safe_path.to_path_buf(),
                        details: format!("header decode failed (too large or invalid): {}", e),
                    });
                }
                continue;
            }
        }
    }

    Err(Error::InvalidFormat {
        path: safe_path.to_path_buf(),
        details: "unexpected EOF while reading header".to_string(),
    })
}
