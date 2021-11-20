pub use crate::ScanError;
pub use crate::{Scan, ScanBinary, ScanLowerHex, ScanOctal, ScanUpperHex};

pub fn advance<'a>(source: &'a str, literal: &str) -> Result<&'a str, ScanError> {
    if source.len() < literal.len() {
        return Err(ScanError::LiteralNotFound);
    }

    if source.is_char_boundary(literal.len()) {
        let (eq_literal, rest) = source.split_at(literal.len());

        if eq_literal == literal {
            Ok(rest)
        } else {
            Err(ScanError::LiteralMismatch)
        }
    } else {
        Err(ScanError::LiteralMismatch)
    }
}
