use crate::container::SafeHeader;
use crate::error::Error;
use std::{fs::File, io::{Read, Write}, path::{Path, PathBuf}};
use crate::consts;

pub struct SafeInfo {
    pub version: u8,
    pub timestamp: u64,
    pub label: String,
    pub ciphertext_len: usize,
}

pub fn write_header<W: Write>(
    w: &mut W,
    header: &SafeHeader,
) -> Result<(), Error> {
    let encoded =
        bincode::serde::encode_to_vec(header, bincode::config::standard())?;

    let len = encoded.len() as u32;

    w.write_all(consts::MAGIC)?;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(&encoded)?;

    Ok(())
}

pub fn read_header<R: Read>(
    r: &mut R,
    path: PathBuf,
) -> Result<(SafeHeader, u32), Error> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != consts::MAGIC {
        return Err(Error::InvalidMagic { path });
    }

    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let header_len = u32::from_le_bytes(len_buf) as usize;

    if header_len > 1024 * 1024 {
        return Err(Error::InvalidFormat {
            path,
            details: "header too large".into(),
        });
    }

    let mut header_buf = vec![0u8; header_len];
    r.read_exact(&mut header_buf)?;

    let (header, _): (SafeHeader, usize) =
        bincode::serde::decode_from_slice(
            &header_buf,
            bincode::config::standard(),
        )?;
    
    if header.version != consts::VERSION {
        return Err(Error::UnsupportedVersion {
            path,
            version: header.version,
        });
    }

    Ok((header, header_len as u32))
}

pub fn inspect_safe_from_path(safe_path: &Path) -> Result<SafeInfo, Error> {
    let mut f = File::open(safe_path).map_err(|e| Error::Io {
        path: Some(safe_path.to_path_buf()),
        source: e,
    })?;

    let (header, _hdr_len) = crate::format::read_header(&mut f, safe_path.to_path_buf())?;

    Ok(SafeInfo {
        version: header.version,
        timestamp: header.timestamp,
        label: header.label,
        ciphertext_len: header.ciphertext_len,
    })
}

