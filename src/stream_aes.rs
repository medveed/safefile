use crate::error::Error;
use aes_gcm_stream::{
    Aes256GcmStreamEncryptor,
    Aes256GcmStreamDecryptor,
};
use colored::Colorize;

pub struct Encryptor {
    inner: Aes256GcmStreamEncryptor,
}

impl Encryptor {
    pub fn new(key: [u8; 32], nonce: &[u8]) -> Self {
        Self {
            inner: Aes256GcmStreamEncryptor::new(key, nonce),
        }
    }

    pub fn update(&mut self, chunk: &[u8]) -> Vec<u8> {
        self.inner.update(chunk)
    }

    pub fn finalize(&mut self) -> (Vec<u8>, [u8; 16]) {
        let (last, tag) = self.inner.finalize();
        let mut tag_arr = [0u8; 16];
        tag_arr.copy_from_slice(&tag);
        (last, tag_arr)
    }
}

pub struct Decryptor {
    inner: Aes256GcmStreamDecryptor,
}

impl Decryptor {
    pub fn new(key: [u8; 32], nonce: &[u8]) -> Self {
        Self {
            inner: Aes256GcmStreamDecryptor::new(key, nonce),
        }
    }

    pub fn update(&mut self, chunk: &[u8]) -> Vec<u8> {
        self.inner.update(chunk)
    }

    pub fn finalize(&mut self) -> Result<Vec<u8>, Error> {
        self.inner
            .finalize()
            .map_err(|e| {
                println!("Decryption finalize error: {}", e.to_string().red());
                Error::InvalidAuthenticationTag
            })
    }
}
