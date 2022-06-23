#[derive(Debug)]
pub enum Errors {
    NoData,
    BufferTooSmall,
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
