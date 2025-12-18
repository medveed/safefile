use std::fmt;
use std::path::PathBuf;

use crate::consts;

#[derive(Debug)]
pub enum Error {
    Io {
        path: Option<PathBuf>,
        source: std::io::Error,
    },
    NotFound {
        path: PathBuf,
    },
    PermissionDenied {
        path: PathBuf,
    },

    InvalidMagic {
        path: PathBuf,
    },
    InvalidFormat {
        path: PathBuf,
        details: String,
    },
    UnsupportedVersion {
        path: PathBuf,
        version: u8,
    },
    IncompleteFile {
        path: PathBuf,
    },

    InvalidAuthenticationTag,

    NotEnoughShares {
        provided: u8,
        required: u8,
    },
    SharingFailed {
        details: String,
    },
    #[allow(unused)]
    ShareCorrupted {
        path: PathBuf,
    },
    ShareChecksumMismatch {
        path: PathBuf,
    },
    ShareVerificationFailed {
        details: String,
    },
    #[allow(unused)]
    OtherShareReconstructionError {
        details: String,
    },

    InternalError {
        details: String,
    },
    InvalidArgument {
        details: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io {
                path: Some(p),
                source,
            } => {
                write!(f, "I/O error on '{}': {}", p.display(), source)
            }
            Error::Io { path: None, source } => {
                write!(f, "I/O error: {}", source)
            }
            Error::NotFound { path } => write!(f, "File '{}' not found", path.display()),
            Error::PermissionDenied { path } => {
                write!(f, "Permission denied: '{}'", path.display())
            }

            Error::InvalidMagic { path } => {
                write!(f, "'{}' is not a valid safefile", path.display())
            }
            Error::InvalidFormat { path, details } => write!(
                f,
                "File '{}' has invalid format: {}",
                path.display(),
                details
            ),
            Error::UnsupportedVersion { path, version } => write!(
                f,
                "Unsupported safefile version {} in '{}' (current version: {})",
                version,
                path.display(),
                consts::VERSION
            ),
            Error::IncompleteFile { path } => {
                write!(f, "File '{}' is truncated or incomplete", path.display())
            }

            Error::InvalidAuthenticationTag => {
                write!(f, "Decryption failed: authentication tag mismatch")
            }

            Error::NotEnoughShares { provided, required } => write!(
                f,
                "Not enough key shares: provided {}, required {}",
                provided, required
            ),
            Error::ShareCorrupted { path } => {
                write!(f, "Key share '{}' is corrupted", path.display())
            }
            Error::ShareChecksumMismatch { path } => write!(
                f,
                "Key share '{}' failed checksum verification",
                path.display()
            ),
            Error::SharingFailed { details } => write!(f, "Secret sharing failed: {}", details),
            Error::ShareVerificationFailed { details } => {
                write!(f, "Key share verification failed: {}", details)
            }
            Error::OtherShareReconstructionError { details } => {
                write!(f, "Share reconstruction error: {}", details)
            }

            Error::InternalError { details } => write!(f, "Internal error: {}", details),
            Error::InvalidArgument { details } => {
                write!(f, "Invalid command line argument: {}", details)
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Error::NotFound {
                path: PathBuf::from(e.to_string()),
            },
            std::io::ErrorKind::PermissionDenied => Error::PermissionDenied {
                path: PathBuf::from(e.to_string()),
            },
            _ => Error::Io {
                path: None,
                source: e,
            },
        }
    }
}

impl From<bincode::error::EncodeError> for Error {
    fn from(e: bincode::error::EncodeError) -> Self {
        Error::InternalError {
            details: format!("bincode encode error: {}", e),
        }
    }
}

impl From<bincode::error::DecodeError> for Error {
    fn from(e: bincode::error::DecodeError) -> Self {
        Error::InternalError {
            details: format!("bincode decode error: {}", e),
        }
    }
}
