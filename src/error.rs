//! Defines the error data containers that may be used by this crate

/// The errors that may be raised by this crate
#[derive(Debug)]
pub enum Errors {
    /// No data was provided despite some being required
    NoData,
    /// The buffer used for an operation was too small
    BufferTooSmall,
    /// The buffer used for an operation was malformed or corrupted
    /// and did not match the expected content constraints
    InvalidBuffer,
}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::NoData => write!(f, "No data was loaded or provided"),
            Errors::BufferTooSmall => write!(f, "The provided data buffer was too small to be used"),
            Errors::InvalidBuffer => write!(f, "The provided data buffer did not provide usable data"),
        }
    }
}
