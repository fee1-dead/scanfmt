use std::{error::Error, fmt::Display};

pub use scanfmt_macros::scanfmt;

#[cfg(test)]
extern crate self as scanfmt;

mod impl_;
pub mod macro_support;

#[cfg(test)]
mod tests;

#[non_exhaustive]
#[derive(Debug)]
pub enum ScanError {
    LiteralMismatch,
    LiteralNotFound,
    Eof,
    Custom(Box<dyn Error + Send + Sync>),
}

/// A trait for something that can be scanned.
pub trait Scan: Sized {
    /// Test if the given character is a valid start for the item to scan.
    fn is_valid_start(c: char) -> bool;
    fn scan(s: &str) -> Result<Self, ScanError>;
}

/// A trait for something that can be scanned in octal format.
pub trait ScanOctal: Sized {
    fn is_valid_start(c: char) -> bool;
    fn scan(s: &str) -> Result<Self, ScanError>;
}

/// A trait for something that can be scanned in binary format, like `fmt::Binary`.
pub trait ScanBinary: Sized {
    fn is_valid_start(c: char) -> bool;
    fn scan(s: &str) -> Result<Self, ScanError>;
}

pub trait ScanLowerHex: Sized {
    fn is_valid_start(c: char) -> bool;
    fn scan(s: &str) -> Result<Self, ScanError>;
}

pub trait ScanUpperHex: Sized {
    fn is_valid_start(c: char) -> bool;
    fn scan(s: &str) -> Result<Self, ScanError>;
}

impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => f.write_str("reached end of sequence while parsing"),
            Self::LiteralMismatch => f.write_str("literal mismatch"),
            Self::LiteralNotFound => f.write_str("literal was not found"),
            Self::Custom(c) => c.fmt(f),
        }
    }
}

impl Error for ScanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Custom(c) = self {
            Some(c.as_ref())
        } else {
            None
        }
    }
}
