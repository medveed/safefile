use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    time::{Duration, Instant},
};

use crate::{container::SafeHeader, error::Error, format, stream_aes};

const REPORT_INTERVAL: Duration = Duration::from_millis(200);

pub fn encrypt_stream<F>(
    input: &Path,
    output: &Path,
    key: [u8; 32],
    nonce: [u8; 12],
    header: SafeHeader,
    mut progress_callback: F,
) -> Result<(), Error>
where
    F: FnMut(u64, u64),
{
    let mut reader = BufReader::new(File::open(input)?);
    let mut writer = BufWriter::new(File::create(output)?);

    let file_size = reader.get_ref().metadata()?.len();
    let mut processed: u64 = 0;
    let mut last_report = Instant::now();

    format::write_header(&mut writer, &header)?;

    let mut enc = stream_aes::Encryptor::new(key, &nonce);
    let mut buf = vec![0u8; 1024 * 1024];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }

        let ct = enc.update(&buf[..n]);
        writer.write_all(&ct)?;

        processed += n as u64;

        if last_report.elapsed() >= REPORT_INTERVAL || processed == file_size {
            progress_callback(processed, file_size);
            last_report = Instant::now();
        }
    }

    let (last_block, tag) = enc.finalize();
    writer.write_all(&last_block)?;
    writer.write_all(&tag)?;
    writer.flush()?;

    Ok(())
}

pub fn decrypt_stream<F>(
    input: &Path,
    output: &Path,
    key: [u8; 32],
    mut progress_callback: F,
) -> Result<(), Error>
where
    F: FnMut(u64, u64),
{
    let mut reader = BufReader::new(File::open(input)?);
    let mut writer = BufWriter::new(File::create(output)?);

    let (header, header_len) = format::read_header(&mut reader, input.into())?;
    let nonce = header.nonce;

    let file_size = reader.get_ref().metadata()?.len();
    let data_start = 8 + header_len as u64; // magic (4) + header length (4) + header bytes
    let total_crypto_len = header.ciphertext_len as u64 + 16; // ciphertext + tag

    if file_size != data_start + total_crypto_len {
        return Err(Error::InvalidFormat {
            path: input.to_path_buf(),
            details: "file size mismatch".into(),
        });
    }

    let mut dec = stream_aes::Decryptor::new(key, &nonce);

    let mut remaining = total_crypto_len;
    let mut buf = vec![0u8; 1024 * 1024];
    let mut processed: u64 = 0;
    let mut last_report = Instant::now();

    while remaining > 0 {
        let to_read = remaining.min(buf.len() as u64) as usize;
        let n = reader.read(&mut buf[..to_read])?;
        if n == 0 {
            return Err(Error::IncompleteFile {
                path: input.to_path_buf(),
            });
        }

        let pt = dec.update(&buf[..n]);
        writer.write_all(&pt)?;
        remaining -= n as u64;
        processed += n as u64;

        if last_report.elapsed() >= REPORT_INTERVAL || remaining == 0 {
            progress_callback(processed, total_crypto_len);
            last_report = Instant::now();
        }
    }

    let last_block = dec.finalize()?;
    writer.write_all(&last_block)?;
    writer.flush()?;

    Ok(())
}
